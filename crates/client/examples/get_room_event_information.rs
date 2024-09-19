// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_client::{Client, OpenTalkApiClient};
use opentalk_client_shared::ApiError;
use opentalk_types_common::rooms::invite_codes::InviteCode;
use snafu::whatever;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), ApiError<reqwest::Error>> {
    const URL_ENV_VAR: &str = "OPENTALK_CONTROLLER_URL";
    const INVITE_CODE_ENV_VAR: &str = "OPENTALK_INVITE_CODE";

    let url_str = whatever!(
        std::env::var(URL_ENV_VAR),
        "Please set the {URL_ENV_VAR} environment variable to the URL of the OpenTalk controller"
    );
    let url = Url::parse(&url_str)?;
    let client = Client::new(url);

    let invite_code_str = whatever!(
        std::env::var(INVITE_CODE_ENV_VAR),
        "Please set the {INVITE_CODE_ENV_VAR} environment variable to the invite code of a room"
    );
    let invite_code = whatever!(
        invite_code_str.parse::<InviteCode>(),
        "Failed to parse invite code {invite_code_str}"
    );

    let verification = client.post_invite_verify(invite_code).await?;

    println!("Verification: {verification:?}");

    let room_event = client
        .get_room_event(invite_code, verification.room_id)
        .await?;

    println!("Event: {room_event:?}");

    Ok(())
}
