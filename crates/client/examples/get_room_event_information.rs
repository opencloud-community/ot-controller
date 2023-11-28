// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use opentalk_client::{Client, OpenTalkApiClient};
use types::core::InviteCodeId;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const URL_ENV_VAR: &str = "OPENTALK_CONTROLLER_URL";
    const INVITE_CODE_ENV_VAR: &str = "OPENTALK_INVITE_CODE";

    let url = Url::parse(
        std::env::var(URL_ENV_VAR)
            .context(format!(
                "Please set the {URL_ENV_VAR} environment variable to the URL of the OpenTalk controller"
            ))?
            .as_str(),
    )?;
    let client = Client::new(url);

    let invite_code = std::env::var(INVITE_CODE_ENV_VAR)
        .context(format!(
            "Please set the {INVITE_CODE_ENV_VAR} environment variable to the invite code of a room"
        ))?
        .parse::<InviteCodeId>()?;

    let verification = client.post_invite_verify(invite_code).await?;

    println!("Verfification: {verification:?}");

    let room_event = client
        .get_room_event(invite_code, verification.room_id)
        .await?;

    println!("Event: {room_event:?}");

    Ok(())
}
