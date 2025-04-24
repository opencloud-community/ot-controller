// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

pub const DEFAULT_MIN_CONNECTIONS: u32 = 10;
pub const DEFAULT_MAX_CHANNELS_PER_CONNECTION: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RabbitMq {
    /// The URL of the RabbitMQ service.
    pub url: String,

    /// The minimum number of connections to be held in the connection pool.
    pub min_connections: u32,

    /// The maximum number of channels per connection.
    pub max_channels_per_connection: u32,

    /// Sending mails will only be enabled if this is set.
    pub mail_task_queue: Option<String>,

    /// Recording will only be enabled if this is set.
    pub recording_task_queue: Option<String>,
}

impl From<settings_file::RabbitMqConfig> for RabbitMq {
    fn from(
        settings_file::RabbitMqConfig {
            url,
            min_connections,
            max_channels_per_connection,
            mail_task_queue,
            recording_task_queue,
        }: settings_file::RabbitMqConfig,
    ) -> Self {
        Self {
            url: url.unwrap_or_else(default_url),
            min_connections: min_connections.unwrap_or(DEFAULT_MIN_CONNECTIONS),
            max_channels_per_connection: max_channels_per_connection
                .unwrap_or(DEFAULT_MAX_CHANNELS_PER_CONNECTION),
            mail_task_queue,
            recording_task_queue,
        }
    }
}

fn default_url() -> String {
    "amqp://guest:guest@localhost:5672".to_string()
}
