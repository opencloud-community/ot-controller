// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeMap, path::Path, sync::Arc};

use bytes::Bytes;
use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;
use either::Either;
use futures::{stream, StreamExt as _, TryStreamExt};
use opentalk_database::Db;
use opentalk_db_storage::{events::Event as DbEvent, users::User};
use opentalk_report_generation::ToReportDateTime;
use opentalk_signaling_core::{
    assets::{save_asset, AssetError, NewAssetFileName},
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
use opentalk_types_common::{
    assets::{asset_file_kind, AssetFileKind, FileExtension},
    modules::ModuleId,
    time::{TimeZone, Timestamp},
    users::UserId,
};
use opentalk_types_signaling::{ParticipantId, ParticipationKind, Role};
use opentalk_types_signaling_meeting_report::{
    command::MeetingReportCommand,
    event::{Error, MeetingReportEvent, PdfAsset},
    MODULE_ID,
};
use snafu::{Report, ResultExt};
use storage::MeetingReportStorage;
use template::{ReportParticipant, ReportTemplateParameter};

pub mod exchange;
mod storage;
mod template;

const DEFAULT_TEMPLATE: &str = include_str!("attendance_report.typ");

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
}

#[async_trait::async_trait(?Send)]
impl SignalingModule for MeetingReport {
    const NAMESPACE: ModuleId = MODULE_ID;

    type Params = ();

    type Incoming = MeetingReportCommand;

    type Outgoing = MeetingReportEvent;

    type ExchangeMessage = exchange::Event;

    type ExtEvent = ();

    type FrontendData = ();

    type PeerFrontendData = ();

    async fn init(
        ctx: InitContext<'_, Self>,
        _params: &Self::Params,
        _protocol: &'static str,
    ) -> Result<Option<Self>, SignalingModuleError> {
        Ok(Some(Self {
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
        _init: SignalingModuleInitData,
    ) -> Result<Option<Self::Params>, SignalingModuleError> {
        Ok(Some(()))
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
        let (participants, event, timezone) = self
            .collect_report_information(&mut ctx, include_email_addresses)
            .await?;

        let report =
            Self::generate_pdf_report(DEFAULT_TEMPLATE.to_string(), event, participants, timezone)
                .await
                .with_whatever_context::<_, _, SignalingModuleError>(|_| {
                    ctx.ws_send(Error::Generate);
                    "Failed to create pdf"
                })?;
        self.upload_pdf(report, ctx).await;

        Ok(())
    }

    async fn collect_report_information(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        include_email_addresses: bool,
    ) -> Result<(Vec<ReportParticipant>, DbEvent, TimeZone), SignalingModuleError> {
        const CONCURRENT_PARTICIPANT_QUERIES: usize = 10;

        let mut conn = self.db.get_conn().await?;
        let storage = ctx.volatile.storage();

        let event = DbEvent::get_for_room(&mut conn, self.room_id.room_id())
            .await?
            .ok_or(SignalingModuleError::NotFoundError {
                message: "Room not found".to_string(),
            })?;

        let event_creator = User::get(&mut conn, event.created_by).await?;
        let timezone = event_creator.timezone.unwrap_or(TimeZone::from(Tz::UTC));
        let tz = Tz::from(timezone);

        // Query all participant IDs and create and concurrently fetch all participant information.
        let participants = storage.get_all_participants(self.room_id).await?;
        let participants = participants.iter().map(|p| async {
            let mut volatile = ctx.volatile.clone();
            let storage = volatile.storage();
            self.query_participant_report(storage, *p, include_email_addresses, &tz)
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

        Ok((participants, event, timezone))
    }

    async fn generate_pdf_report(
        template: String,
        event: DbEvent,
        participants: Vec<ReportParticipant>,
        report_timezone: TimeZone,
    ) -> Result<Vec<u8>, SignalingModuleError> {
        let tz = Tz::from(report_timezone);
        let starts_at = event.starts_at.to_report_date_time(&tz);
        let ends_at = event.ends_at.to_report_date_time(&tz);
        let timestamp = Local::now().naive_local().format("%Y-%m-%dT%H:%M:%S.%f");
        Self::generate_pdf_report_from_template(
            template,
            &ReportTemplateParameter {
                title: event.title,
                description: event.description,
                starts_at,
                ends_at,
                report_timezone,
                participants,
            },
            Path::new(&format!("{MODULE_ID}/{timestamp}")),
        )
    }

    fn generate_pdf_report_from_template(
        template: String,
        parameter: &ReportTemplateParameter,
        dump_to_relative_path: &Path,
    ) -> Result<Vec<u8>, SignalingModuleError> {
        let dump_to_path = std::env::var("OPENTALK_REPORT_DUMP_PATH")
            .map(|p| Path::new(&p).join(dump_to_relative_path))
            .ok();

        let pdf = opentalk_report_generation::generate_pdf_report(
            template,
            BTreeMap::from_iter([(
                Path::new("data.json"),
                serde_json::to_string_pretty(parameter)
                    .unwrap()
                    .into_bytes()
                    .into(),
            )]),
            dump_to_path.as_deref(),
        )
        .whatever_context::<_, SignalingModuleError>("unable to build pdf")?;
        Ok(pdf)
    }

    async fn upload_pdf(&mut self, report: Vec<u8>, mut ctx: ModuleContext<'_, Self>) {
        const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("meeting_report");
        let file_name =
            NewAssetFileName::new(ASSET_FILE_KIND, Timestamp::now(), FileExtension::pdf());
        let report =
            async_stream::stream!(yield Result::<_, ObjectStorageError>::Ok(Bytes::from(report)));
        let report = Box::pin(report);
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
        tz: &Tz,
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
                    .get_local(JOINED_AT)
                    .get_local(LEFT_AT)
                    .get_global(DISPLAY_NAME)
                    .get_global(ROLE)
                    .get_local(KIND)
                    .get_local(USER_ID),
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

        // The report generator does not support sub-seconds resolution, so we
        // round the timestamps to seconds
        let joined_at = joined_at.to_report_date_time(tz);
        let left_at = left_at.to_report_date_time(tz);

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use insta::assert_snapshot;

    use crate::{template::ReportTemplateParameter, MeetingReport, DEFAULT_TEMPLATE, MODULE_ID};

    fn generate(sample_name: &str, parameter: &ReportTemplateParameter) -> String {
        let pdf = MeetingReport::generate_pdf_report_from_template(
            DEFAULT_TEMPLATE.to_string(),
            parameter,
            Path::new(&format!("{MODULE_ID}/{sample_name}")),
        )
        .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn generate_report_small() {
        assert_snapshot!(generate("small", &crate::template::tests::example_small()), @r#"
        Attendance Report
         Meeting : Testmeeting

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator
        "#);
    }

    #[test]
    fn generate_report_medium() {
        assert_snapshot!(
            generate(
                "medium",
                &crate::template::tests::example_medium()
            ),
            @r#"
        Attendance Report
         Meeting : Testmeeting

        Details : A medium sized test meeting

        Start : 2025-02-06 08:18

        End : 2025-02-06 11:25

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator

        2 Charlie Cooper User

        3 Bob Burton User
        "#
        );
    }

    #[test]
    fn generate_report_large() {
        assert_snapshot!(generate("large", &crate::template::tests::example_large()), @r#"
        Attendance Report
         Meeting : Large Testmeeting

        Details : The large test meeting

        Start : 2025-02-06 08:18

        End : 2025-02-06 11:25

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice Adams Moderator

        2 Franz Fischer User

        3 Charlie Cooper User

        4 Bob Burton User

        5 Erin Guest

        6 Dave Dunn Guest
        "#);
    }
}
