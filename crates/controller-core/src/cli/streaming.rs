// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::map_set_clear_to_option_option;
use anyhow::{Context, Result};
use clap::Subcommand;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use opentalk_controller_settings::Settings;
use opentalk_database::Db;
use opentalk_db_storage::streaming_services::{
    NewStreamingService, StreamingServiceRecord, UpdateStreamingService,
};
use opentalk_types::common::streaming::StreamingServiceKind;
use opentalk_types::core::{StreamingKind, StreamingServiceId};
use tabled::{settings::Style, Table, Tabled};
use url::Url;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List all available streaming services
    List,
    /// Create a new streaming service
    Create {
        /// The name of the streaming service
        name: String,
        /// The kind of the streaming service
        kind: StreamingServiceKind,
        /// The endpoint url of the streaming service (for `provider` kind only)
        streaming_url: Option<Url>,
        /// The format of the streaming key (for `provider` kind only)
        streaming_key_regex: Option<String>,
        /// The format of the url from which the stream can be accessed (for `provider` kind only)
        public_url_regex: Option<String>,
    },
    /// Delete a streaming service by id
    Delete {
        /// The Id of the streaming service to delete
        id: StreamingServiceId,
    },
    /// Modify an existing streaming service
    Edit {
        /// The Id of the streaming service to modify
        id: StreamingServiceId,
        /// Set a new name
        #[clap(long)]
        set_name: Option<String>,
        /// Set the kind
        #[clap(long)]
        set_kind: Option<StreamingServiceKind>,
        /// Set the endpoint url (for `provider` kind only)
        #[clap(long)]
        set_streaming_url: Option<Url>,
        /// Set the format of the streaming key (for `provider` kind only)
        #[clap(long)]
        set_streaming_key_regex: Option<String>,
        /// Set the format of the url from which the stream can be accessed (for `provider` kind only)
        #[clap(long)]
        set_public_url_regex: Option<String>,
    },
}

pub async fn handle_command(settings: Settings, command: Command) -> Result<()> {
    match command {
        Command::List => list_all_streaming_services(settings).await,
        Command::Create {
            name,
            kind,
            streaming_url,
            streaming_key_regex,
            public_url_regex,
        } => {
            create_streaming_service(
                settings,
                name,
                kind,
                streaming_url,
                streaming_key_regex,
                public_url_regex,
            )
            .await
        }
        Command::Delete { id } => delete_streaming_service(settings, id).await,
        Command::Edit {
            id,
            set_name,
            set_kind,
            set_streaming_url,
            set_streaming_key_regex,
            set_public_url_regex,
        } => {
            edit_streaming_service(
                settings,
                id,
                set_name,
                set_kind,
                set_streaming_url,
                set_streaming_key_regex,
                set_public_url_regex,
            )
            .await
        }
    }
}

#[derive(Tabled)]
struct StreamingServiceTableRow {
    id: StreamingServiceId,
    name: String,
    kind: String,
    streaming_url: String,
    streaming_key_regex: String,
    public_url_regex: String,
}

impl StreamingServiceTableRow {
    fn from_streaming_service(streaming_service: StreamingServiceRecord) -> Self {
        Self {
            id: streaming_service.id,
            name: streaming_service.name,
            kind: streaming_service.kind.to_string(),
            streaming_url: streaming_service.streaming_url.unwrap_or("".to_string()),
            streaming_key_regex: streaming_service
                .streaming_key_regex
                .unwrap_or("".to_string()),
            public_url_regex: streaming_service.public_url_regex.unwrap_or("".to_string()),
        }
    }
}

/// Implementation of the `opentalk-controller streaming-services list` command
async fn list_all_streaming_services(settings: Settings) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    let streaming_services = StreamingServiceRecord::get_all(&mut conn).await?;

    print_streaming_services(streaming_services).await
}

async fn create_streaming_service(
    settings: Settings,
    name: String,
    kind: StreamingServiceKind,
    streaming_url: Option<Url>,
    streaming_key_regex: Option<String>,
    public_url_regex: Option<String>,
) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    conn.transaction(|conn| {
        async move {
            let streaming_service = NewStreamingService {
                name: name.clone(),
                kind: map_external_to_db_kind(kind),
                streaming_url: streaming_url.map(|s| s.into()),
                streaming_key_regex,
                public_url_regex,
            }
            .insert(conn)
            .await?;

            println!(
                "Created streaming service {name} ({})",
                streaming_service.id
            );

            Ok(())
        }
        .scope_boxed()
    })
    .await
}

async fn delete_streaming_service(settings: Settings, id: StreamingServiceId) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    conn.transaction(|conn| {
        async move {
            let service_name = StreamingServiceRecord::get(conn, id).await?.name;
            StreamingServiceRecord::delete_by_id(conn, id).await?;

            println!("Deleted streaming service {service_name} ({id})");

            Ok(())
        }
        .scope_boxed()
    })
    .await
}

#[allow(clippy::too_many_arguments)]
async fn edit_streaming_service(
    settings: Settings,
    id: StreamingServiceId,
    set_name: Option<String>,
    set_kind: Option<StreamingServiceKind>,
    set_streaming_url: Option<Url>,
    set_streaming_key_regex: Option<String>,
    set_public_url_regex: Option<String>,
) -> Result<()> {
    let db = Db::connect(&settings.database).context("Failed to connect to database")?;
    let mut conn = db.get_conn().await?;

    let clear_provider_fields = set_kind
        .as_ref()
        .map_or(false, |s| !matches!(s, StreamingServiceKind::Provider));

    conn.transaction(|conn| {
        async move {
            let service_name = StreamingServiceRecord::get(conn, id).await?.name;

            let streaming_url =
                map_set_clear_to_option_option(set_streaming_url, clear_provider_fields);
            let streaming_key_regex =
                map_set_clear_to_option_option(set_streaming_key_regex, clear_provider_fields);
            let public_url_regex =
                map_set_clear_to_option_option(set_public_url_regex, clear_provider_fields);

            // Apply changeset
            let streaming_service = UpdateStreamingService {
                name: set_name,
                kind: set_kind.map(map_external_to_db_kind),
                streaming_url: streaming_url.map(|s| s.map(|s| s.into())),
                streaming_key_regex,
                public_url_regex,
            }
            .apply(conn, id)
            .await?;

            println!("Updated streaming service {} ({})", service_name, id);
            print_streaming_services([streaming_service]).await
        }
        .scope_boxed()
    })
    .await
}

fn map_external_to_db_kind(kind: StreamingServiceKind) -> StreamingKind {
    match kind {
        StreamingServiceKind::Builtin => StreamingKind::Builtin,
        StreamingServiceKind::Custom => StreamingKind::Custom,
        StreamingServiceKind::Provider => StreamingKind::Provider,
    }
}

async fn print_streaming_services(
    streaming_services: impl IntoIterator<Item = StreamingServiceRecord>,
) -> Result<()> {
    let rows: Vec<StreamingServiceTableRow> = streaming_services
        .into_iter()
        .map(StreamingServiceTableRow::from_streaming_service)
        .collect();

    println!("{}", Table::new(rows).with(Style::ascii()));

    Ok(())
}
