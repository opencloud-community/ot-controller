// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! MailService
//!
//! Used to have a clean interface for various kinds of mails
//! that are sent from the Web-API and possibly other connected services.
//!
// TODO: We probably can avoid the conversion to MailTasks if no rabbit_mq_queue is set in all mail fns
use std::sync::Arc;

use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use opentalk_controller_settings::Settings;
use opentalk_db_storage::{
    events::{Event, EventException, EventExceptionKind},
    rooms::Room,
    sip_configs::SipConfig,
    users::User,
};
use opentalk_mail_worker_protocol::*;
use opentalk_types_common::{
    features,
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    users::{Language, UserId, UserTitle},
};
use snafu::ResultExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{metrics::EndpointMetrics, Result};

/// A registered mail recipient
#[derive(Debug)]
pub struct RegisteredMailRecipient {
    /// The user id
    pub id: UserId,
    /// The email address
    pub email: String,
    /// The title
    pub title: UserTitle,
    /// The first name
    pub first_name: String,
    /// The last name
    pub last_name: String,
    /// The language
    pub language: Language,
}

impl From<User> for RegisteredMailRecipient {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            title: user.title,
            first_name: user.firstname,
            last_name: user.lastname,
            language: user.language,
        }
    }
}

/// An unregistered mail recipient
#[derive(Debug)]
pub struct UnregisteredMailRecipient {
    /// The email address
    pub email: String,
    /// The first name
    pub first_name: String,
    /// The last name
    pub last_name: String,
}

/// An external mail recipient
#[derive(Debug)]
pub struct ExternalMailRecipient {
    /// The email address
    pub email: String,
}

/// A mail recipient
#[derive(Debug)]
pub enum MailRecipient {
    /// A registered mail recipient
    Registered(RegisteredMailRecipient),
    /// An unregistered mail recipient
    Unregistered(UnregisteredMailRecipient),
    /// An external mail recipient
    External(ExternalMailRecipient),
}

fn to_event(
    settings: &Settings,
    event: Event,
    room: Room,
    sip_config: Option<SipConfig>,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> v1::Event {
    const ONE_DAY_IN_SECONDS: u64 = 86400;

    let created_at = v1::Time {
        time: event.created_at,
        timezone: event.created_at.timezone().to_string(),
    };

    let start_time: Option<v1::Time> = event.starts_at.zip(event.starts_at_tz).map(Into::into);

    let end_time: Option<v1::Time> = event.ends_at_of_first_occurrence().map(Into::into);

    let call_in_feature_is_enabled = !settings
        .raw
        .defaults
        .disabled_features
        .contains(&features::CALL_IN_MODULE_FEATURE_ID);

    let mut call_in = None;

    if call_in_feature_is_enabled {
        if let (Some(call_in_settings), Some(sip_config)) = (&settings.raw.call_in, sip_config) {
            call_in = Some(v1::CallIn {
                sip_tel: call_in_settings.tel.clone(),
                sip_id: sip_config.sip_id.to_string(),
                sip_password: sip_config.password.to_string(),
            });
        }
    }

    let adhoc_retention_seconds = if event.is_adhoc {
        Some(ONE_DAY_IN_SECONDS)
    } else {
        None
    };

    v1::Event {
        id: Uuid::from(event.id),
        name: event.title,
        description: event.description,
        created_at,
        start_time,
        end_time,
        rrule: event.recurrence_pattern,
        room: v1::Room {
            id: Uuid::from(room.id),
            password: room.password,
        },
        call_in,
        revision: event.revision,
        shared_folder,
        adhoc_retention_seconds,
        streaming_targets,
    }
}

fn to_event_exception(exception: EventException) -> v1::EventException {
    let exception_date = v1::Time {
        time: exception.exception_date,
        timezone: exception.exception_date_tz.to_string(),
    };

    let kind = match exception.kind {
        EventExceptionKind::Modified => v1::EventExceptionKind::Modified,
        EventExceptionKind::Cancelled => v1::EventExceptionKind::Canceled,
    };

    let starts_at: Option<v1::Time> = exception
        .starts_at
        .zip(exception.starts_at_tz)
        .map(Into::into);

    let ends_at: Option<v1::Time> = exception.ends_at.zip(exception.ends_at_tz).map(Into::into);

    v1::EventException {
        exception_date,
        kind,
        title: exception.title,
        description: exception.description,
        is_all_day: exception.is_all_day,
        starts_at,
        ends_at,
    }
}

/// A service for sending emails
#[derive(Clone)]
pub struct MailService {
    metrics: Arc<EndpointMetrics>,
    rabbitmq_pool: Arc<RabbitMqPool>,
    rabbitmq_channel: Arc<Mutex<RabbitMqChannel>>,
}

impl std::fmt::Debug for MailService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MailService")
    }
}

impl MailService {
    /// Creates a new email service
    pub fn new(
        metrics: Arc<EndpointMetrics>,
        rabbitmq_pool: Arc<RabbitMqPool>,
        rabbitmq_channel: RabbitMqChannel,
    ) -> Self {
        Self {
            metrics,
            rabbitmq_pool,
            rabbitmq_channel: Arc::new(Mutex::new(rabbitmq_channel)),
        }
    }

    async fn send_to_rabbitmq(&self, settings: &Settings, mail_task: MailTask) -> Result<()> {
        if let Some(queue_name) = &settings.raw.rabbit_mq.mail_task_queue {
            let channel = {
                let mut channel = self.rabbitmq_channel.lock().await;

                if !channel.status().connected() {
                    // Check if channel is healthy - try to reconnect if it isn't
                    *channel = self
                        .rabbitmq_pool
                        .create_channel()
                        .await
                        .whatever_context("Failed to get a rabbitmq_channel replacement")?;
                }

                channel.clone()
            };

            _ = channel
                .basic_publish(
                    "",
                    queue_name,
                    Default::default(),
                    &serde_json::to_vec(&mail_task)
                        .whatever_context("Failed to serialize mail_task")?,
                    Default::default(),
                )
                .await
                .whatever_context("Failed to publish to channel")?;
        }

        self.metrics.increment_issued_email_tasks_count(&mail_task);

        Ok(())
    }

    /// Sends a Registered Invite mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_registered_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: User,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let shared_folder = shared_folder.map(|sf| {
            if inviter.id == invitee.id {
                sf
            } else {
                sf.without_write_access()
            }
        });

        // Create MailTask
        let mail_task = MailTask::registered_event_invite(
            inviter,
            to_event(
                settings,
                event,
                room,
                sip_config,
                shared_folder,
                streaming_targets,
            ),
            invitee,
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends a Unregistered Invite mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_unregistered_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: opentalk_keycloak_admin::users::User,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let invitee = v1::UnregisteredUser {
            email: invitee.email.into(),
            first_name: invitee.first_name,
            last_name: invitee.last_name,
        };

        // Create MailTask
        let mail_task = MailTask::unregistered_event_invite(
            inviter,
            to_event(
                settings,
                event,
                room,
                sip_config,
                shared_folder.map(SharedFolder::without_write_access),
                streaming_targets,
            ),
            invitee,
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends a external Invite mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_external_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: &str,
        invite_code: String,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        // Create MailTask
        let mail_task = MailTask::external_event_invite(
            inviter,
            to_event(
                settings,
                event,
                room,
                sip_config,
                shared_folder.map(SharedFolder::without_write_access),
                streaming_targets,
            ),
            invitee.to_string(),
            invite_code,
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends an Event Update mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_event_update(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        event_exception: Option<EventException>,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        invite_code: String,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if invitee.id == inviter.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_update(
                    inviter,
                    to_event(
                        settings,
                        event,
                        room,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    event_exception.map(to_event_exception),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_update(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                event_exception.map(to_event_exception),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_update(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                event_exception.map(to_event_exception),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
                invite_code,
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }

    /// Sends an Event Cancellation mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_event_cancellation(
        &self,
        settings: &Settings,
        inviter: User,
        mut event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        // increment event sequence to satisfy icalendar spec
        event.revision += 1;

        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if inviter.id == invitee.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_cancellation(
                    inviter,
                    to_event(
                        settings,
                        event,
                        room,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_cancellation(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_cancellation(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }

    /// Sends an Event Uninvite mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_event_uninvite(
        &self,
        settings: &Settings,
        inviter: User,
        mut event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        // increment event sequence to satisfy icalendar spec
        event.revision += 1;

        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if inviter.id == invitee.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_uninvite(
                    inviter,
                    to_event(
                        settings,
                        event,
                        room,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_uninvite(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_uninvite(
                inviter,
                to_event(
                    settings,
                    event,
                    room,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }
}
