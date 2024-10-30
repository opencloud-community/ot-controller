// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Utc};
use either::Either;
use futures::{stream, Stream, StreamExt as _, TryStreamExt};
use opentalk_controller_settings::ReportsTemplate;
use opentalk_database::Db;
use opentalk_db_storage::{events::Event as DbEvent, users::User};
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, AssetFileKind, NewAssetFileName},
    control::{
        self,
        storage::{
            AttributeActions, ControlStorageParticipantAttributes, DISPLAY_NAME, JOINED_AT, KIND,
            LEFT_AT, ROLE, USER_ID,
        },
    },
    ChunkFormat, DestroyContext, Event, InitContext, ModuleContext, ObjectStorage,
    ObjectStorageError, SignalingModule, SignalingModuleError, SignalingModuleInitData,
    SignalingRoomId, VolatileStorage,
};
use opentalk_types::signaling::meeting_report::event::{Error, MeetingReportEvent};
use opentalk_types_common::{assets::FileExtension, time::Timestamp, users::UserId};
use opentalk_types_signaling::{ParticipantId, ParticipationKind, Role};
use opentalk_types_signaling_meeting_report::{
    command::MeetingReportCommand, event::PdfAsset, NAMESPACE,
};
use snafu::Report;
use storage::MeetingReportStorage;
use template::{ReportParticipant, ReportTemplateParameter};
use terdoc_client::{InputFormat, TaskData, Template, TerdocClient, TerdocStreamingClientTrait};

pub mod exchange;
mod storage;
mod template;

const DEFAULT_TEMPLATE: &str = include_str!("report.md.tera");

trait MeetingReportStorageProvider {
    fn storage(&mut self) -> &mut dyn MeetingReportStorage;
}

impl MeetingReportStorageProvider for VolatileStorage {
    fn storage(&mut self) -> &mut dyn MeetingReportStorage {
        match self.as_mut() {
            Either::Left(v) => v,
            Either::Right(v) => v,
        }
    }
}

pub struct MeetingReport {
    room_id: SignalingRoomId,
    db: Arc<Db>,
    storage: Arc<ObjectStorage>,

    terdoc_client: TerdocClient,
    template: String,
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for MeetingReport {
    const NAMESPACE: &'static str = NAMESPACE;

    type Params = (TerdocClient, String);

    type Incoming = MeetingReportCommand;

    type Outgoing = MeetingReportEvent;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = ();

    type FrontendData = ();

    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        (terdoc_client, template): &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
            terdoc_client: terdoc_client.clone(),
            template: template.clone(),

            room_id: ctx.room_id(),
            db: ctx.db.clone(),
            storage: ctx.storage.clone(),
        }))
    }

    async fn on_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<(), SignalingModuleError> {
        match event {
            Event::Joined { .. } => {}
            Event::WsMessage(msg) => {
                self.handle_ws_message(ctx, msg).await?;
            }
            Event::Exchange(..)
            | Event::Ext(..)
            | Event::ParticipantJoined(..)
            | Event::Leaving
            | Event::RaiseHand
            | Event::LowerHand
            | Event::ParticipantUpdated(_, _)
            | Event::ParticipantLeft(_)
            | Event::RoleUpdated(_) => {}
        }

        Ok(())
    }

    async fn on_destroy(self, _ctx: DestroyContext<'_>) {}

    async fn build_params(
        init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        let Some(report_settings) = init.startup_settings.reports.as_ref() else {
            return Ok(None);
        };

        let client =
            terdoc_client::TerdocClient::new(report_settings.url.as_str()).map_err(|e| {
                SignalingModuleError::CustomError {
                    message: "Failed to initialize terdoc client".to_string(),
                    source: Some(Box::new(e).into()),
                }
            })?;
        let template = match &report_settings.template {
            ReportsTemplate::Inline(template) => template.clone(),
            ReportsTemplate::BuiltIn => DEFAULT_TEMPLATE.to_owned(),
        };

        Ok(Some((client, template)))
    }
}

impl MeetingReport {
    async fn handle_ws_message(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        msg: MeetingReportCommand,
    ) -> Result<(), SignalingModuleError> {
        match msg {
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses,
            } => {
                if ctx.role() != Role::Moderator {
                    ctx.ws_send(Error::InsufficientPermissions);
                    return Ok(());
                }
                self.create_attendance_report(ctx, include_email_addresses)
                    .await?;
            }
        }
        Ok(())
    }

    async fn create_attendance_report(
        &mut self,
        mut ctx: ModuleContext<'_, Self>,
        include_email_addresses: bool,
    ) -> Result<(), SignalingModuleError> {
        let (participants, event) = self
            .collect_report_information(&mut ctx, include_email_addresses)
            .await?;

        // we need to clone the terdoc client here
        let mut td_client = self.terdoc_client.clone();
        let report =
            Self::generate_report_pdf(&mut td_client, self.template.clone(), event, participants)
                .await;

        let report = match report {
            Ok(r) => r,
            e @ Err(_) => {
                ctx.ws_send(Error::Generate);
                e?
            }
        };
        let report = report.map_err(|e| ObjectStorageError::Other {
            message: "Terdoc error".to_string(),
            source: Some(e.into()),
        });

        self.upload_pdf(report, ctx).await;

        Ok(())
    }

    async fn collect_report_information(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        include_email_addresses: bool,
    ) -> Result<(Vec<ReportParticipant>, DbEvent), SignalingModuleError> {
        const CONCURRENT_PARTICIPANT_QUERIES: usize = 10;

        let mut conn = self.db.get_conn().await?;
        let storage = ctx.volatile.storage();

        // Query all participant IDs and create and concurrently fetch all participant information.
        let participants = storage.get_all_participants(self.room_id).await?;
        let participants = participants.iter().map(|p| async {
            let mut volatile = ctx.volatile.clone();
            let storage = volatile.storage();
            self.query_participant_report(storage, *p, include_email_addresses)
                .await
        });

        let participants = stream::iter(participants)
            .buffer_unordered(CONCURRENT_PARTICIPANT_QUERIES)
            .try_collect::<Vec<ReportParticipant>>()
            .await
            .map_err(|e| SignalingModuleError::CustomError {
                message: "Failed to query participants".to_string(),
                source: Some(Box::new(e).into()),
            })?;

        let event = DbEvent::get_for_room(&mut conn, self.room_id.room_id())
            .await?
            .ok_or(SignalingModuleError::NotFoundError {
                message: "Room not found".to_string(),
            })?;

        Ok((participants, event))
    }

    async fn generate_report_pdf(
        terdoc_client: &mut TerdocClient,
        template: String,
        event: DbEvent,
        participants: Vec<ReportParticipant>,
    ) -> Result<
        impl Stream<Item = Result<bytes::Bytes, <TerdocClient as TerdocStreamingClientTrait>::Error>>
            + '_,
        SignalingModuleError,
    > {
        let task_data = TaskData {
            output: terdoc_client::OutputFormat::Pdf {
                engine: terdoc_client::PdfEngine::LibreOffice,
                theme: None,
            },
            data: serde_json::to_value(ReportTemplateParameter {
                title: event.title,
                description: event.description,
                starts_at: event.starts_at,
                starts_at_tz: event.starts_at_tz,
                ends_at: event.ends_at,
                ends_at_tz: event.ends_at_tz,
                participants,
            })
            .expect("TaskData is serializable"),
            template: Template {
                content: template,
                format: InputFormat::Markdown,
                autoescape: false,
            },
        };

        terdoc_client.request_render(task_data).await.map_err(|e| {
            SignalingModuleError::CustomError {
                message: "Render request failed".to_string(),
                source: Some(Box::new(e).into()),
            }
        })
    }

    async fn upload_pdf(
        &mut self,
        report: impl Stream<Item = Result<bytes::Bytes, ObjectStorageError>> + Unpin + '_,
        mut ctx: ModuleContext<'_, Self>,
    ) {
        let asset_file_kind = AssetFileKind::from_str("meeting_report")
            .expect("The asset file kind is at least 1, but no longer than 20 chars and only contains alphanumeric chars and `_`");
        let file_name =
            NewAssetFileName::new(asset_file_kind, Timestamp::now(), FileExtension::pdf());
        let result = save_asset(
            &self.storage,
            self.db.clone(),
            self.room_id.room_id(),
            Some(Self::NAMESPACE),
            file_name,
            report,
            ChunkFormat::Data,
        )
        .await;

        // If storing the asset failed, we report the error and silently return.
        let (asset_id, file_name) = match result {
            Ok(inner) => inner,
            Err(AssetError::AssetStorageExceeded) => {
                log::debug!("Storage exceeded while storing meeting report");
                ctx.ws_send(MeetingReportEvent::Error(Error::StorageExceeded));
                return;
            }
            Err(e) => {
                log::error!(
                    "Error while storing attendance report: {}",
                    Report::from_error(e)
                );
                ctx.ws_send(MeetingReportEvent::Error(Error::Storage));
                return;
            }
        };

        let pdf_asset = PdfAsset {
            filename: file_name,
            asset_id,
        };
        log::debug!("Generated meeting attendance report: {:?}", pdf_asset);
        ctx.exchange_publish(
            control::exchange::current_room_all_participants(self.room_id),
            exchange::Event::PdfAsset(pdf_asset.clone()),
        );
        ctx.ws_send(MeetingReportEvent::PdfAsset(pdf_asset));
    }

    async fn query_participant_report(
        &self,
        storage: &mut dyn MeetingReportStorage,
        participant: ParticipantId,
        include_email_addresses: bool,
    ) -> Result<ReportParticipant, SignalingModuleError> {
        type UserData = (
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            String,
            Role,
            ParticipationKind,
            Option<UserId>,
        );

        let (joined_at, left_at, name, role, kind, user_id): UserData = storage
            .bulk_attribute_actions(
                AttributeActions::new(self.room_id, participant)
                    .get(JOINED_AT)
                    .get(LEFT_AT)
                    .get(DISPLAY_NAME)
                    .get(ROLE)
                    .get(KIND)
                    .get(USER_ID),
            )
            .await?;

        let email = match (include_email_addresses, user_id) {
            (true, Some(user_id)) => {
                let mut conn = self.db.get_conn().await?;
                let user = User::get(&mut conn, user_id).await?;

                Some(user.email)
            }
            _ => None,
        };

        Ok(ReportParticipant {
            id: participant,
            name,
            joined_at,
            left_at,
            role,
            email,
            kind,
        })
    }
}
