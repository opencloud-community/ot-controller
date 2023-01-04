// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! MailService
//!
//! Used to have a clean interface for various kinds of mails
//! that are sent from the Web-API and possibly other connected services.
//!
// TODO We probably can avoid the conversion to MailTasks if no rabbit_mq_queue is set in all mail fns
use crate::metrics::EndpointMetrics;
use anyhow::{Context, Result};
use controller_settings::{Settings, SharedSettings};
use db_storage::{events::Event, rooms::Room, sip_configs::SipConfig, users::User};
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use mail_worker_proto::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::common::{features, shared_folder::SharedFolder};
use types::core::UserId;
use uuid::Uuid;

pub struct RegisteredMailRecipient {
    pub id: UserId,
    pub email: String,
    pub title: String,
    pub first_name: String,
    pub last_name: String,
    pub language: String,
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

pub struct UnregisteredMailRecipient {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

pub struct ExternalMailRecipient {
    pub email: String,
}

pub enum MailRecipient {
    Registered(RegisteredMailRecipient),
    Unregistered(UnregisteredMailRecipient),
    External(ExternalMailRecipient),
}

fn to_event(
    event: Event,
    room: Room,
    sip_config: Option<SipConfig>,
    settings: &Settings,
    shared_folder: Option<SharedFolder>,
) -> mail_worker_proto::v1::Event {
    const ONE_DAY_IN_SECONDS: u64 = 86400;

    let start_time: Option<v1::Time> = event.starts_at.zip(event.starts_at_tz).map(Into::into);

    let end_time: Option<v1::Time> = event.ends_at_of_first_occurrence().map(Into::into);

    let call_in_feature_is_enabled = !settings
        .defaults
        .disabled_features()
        .contains(features::CALL_IN);

    let mut call_in = None;

    if call_in_feature_is_enabled {
        if let (Some(call_in_settings), Some(sip_config)) = (&settings.call_in, sip_config) {
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

    mail_worker_proto::v1::Event {
        id: Uuid::from(event.id),
        name: event.title,
        description: event.description,
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
    }
}

#[derive(Clone)]
pub struct MailService {
    settings: SharedSettings,
    metrics: Arc<EndpointMetrics>,
    rabbitmq_pool: Arc<RabbitMqPool>,
    rabbitmq_channel: Arc<Mutex<RabbitMqChannel>>,
}

impl MailService {
    pub fn new(
        settings: SharedSettings,
        metrics: Arc<EndpointMetrics>,
        rabbitmq_pool: Arc<RabbitMqPool>,
        rabbitmq_channel: RabbitMqChannel,
    ) -> Self {
        Self {
            settings,
            metrics,
            rabbitmq_pool,
            rabbitmq_channel: Arc::new(Mutex::new(rabbitmq_channel)),
        }
    }

    async fn send_to_rabbitmq(&self, mail_task: MailTask) -> Result<()> {
        if let Some(queue_name) = &self.settings.load().rabbit_mq.mail_task_queue {
            let channel = {
                let mut channel = self.rabbitmq_channel.lock().await;

                if !channel.status().connected() {
                    // Check if channel is healthy - try to reconnect if it isn't
                    *channel = self
                        .rabbitmq_pool
                        .create_channel()
                        .await
                        .context("Failed to get a rabbitmq_channel replacement")?;
                }

                channel.clone()
            };

            channel
                .basic_publish(
                    "",
                    queue_name,
                    Default::default(),
                    &serde_json::to_vec(&mail_task).context("Failed to serialize mail_task")?,
                    Default::default(),
                )
                .await?;
        }

        self.metrics.increment_issued_email_tasks_count(&mail_task);

        Ok(())
    }

    /// Sends a Registered Invite mail task to the rabbit mq queue, if configured.
    pub async fn send_registered_invite(
        &self,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: User,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();
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
            to_event(event, room, sip_config, settings, shared_folder),
            invitee,
        );

        self.send_to_rabbitmq(mail_task).await?;
        Ok(())
    }

    /// Sends a Unregistered Invite mail task to the rabbit mq queue, if configured.
    pub async fn send_unregistered_invite(
        &self,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: keycloak_admin::users::User,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();

        // Create MailTask
        let mail_task = MailTask::unregistered_event_invite(
            inviter,
            to_event(
                event,
                room,
                sip_config,
                settings,
                shared_folder.map(SharedFolder::without_write_access),
            ),
            invitee,
        );

        self.send_to_rabbitmq(mail_task).await?;
        Ok(())
    }

    /// Sends a external Invite mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_external_invite(
        &self,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: &str,
        invite_code: String,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();

        // Create MailTask
        let mail_task = MailTask::external_event_invite(
            inviter,
            to_event(
                event,
                room,
                sip_config,
                settings,
                shared_folder.map(SharedFolder::without_write_access),
            ),
            invitee.to_string(),
            invite_code,
        );

        self.send_to_rabbitmq(mail_task).await?;
        Ok(())
    }

    /// Sends an Event Update mail task to the rabbit mq queue, if configured.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_event_update(
        &self,
        inviter: User,
        event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        invite_code: String,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();

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
                    to_event(event, room, sip_config, settings, shared_folder),
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
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
                ),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_update(
                inviter,
                to_event(
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
                invite_code,
            ),
        };

        self.send_to_rabbitmq(mail_task).await?;

        Ok(())
    }

    /// Sends an Event Cancellation mail task to the rabbit mq queue, if configured.
    pub async fn send_event_cancellation(
        &self,
        inviter: User,
        mut event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();

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
                    to_event(event, room, sip_config, settings, shared_folder),
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
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
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
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(mail_task).await?;

        Ok(())
    }

    /// Sends an Event Uninvite mail task to the rabbit mq queue, if configured.
    pub async fn send_event_uninvite(
        &self,
        inviter: User,
        mut event: Event,
        room: Room,
        sip_config: Option<SipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
    ) -> Result<()> {
        let settings = &*self.settings.load();

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
                    to_event(event, room, sip_config, settings, shared_folder),
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
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
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
                    event,
                    room,
                    sip_config,
                    settings,
                    shared_folder.map(SharedFolder::without_write_access),
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(mail_task).await?;

        Ok(())
    }
}
