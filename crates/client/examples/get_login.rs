// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_client::{Client, OpenTalkApiClient};
use opentalk_client_shared::ApiError;
use snafu::whatever;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), ApiError<reqwest::Error>> {
    const URL_ENV_VAR: &str = "OPENTALK_CONTROLLER_URL";

    let url_str = whatever!(
        std::env::var(URL_ENV_VAR),
        "Please set the {URL_ENV_VAR} environment variable to the URL of the OpenTalk controller"
    );
    let url = Url::parse(&url_str)?;
    let client = Client::new(url);

    let oidc_provider = client.get_login().await?;

    println!("OIDC provider: {oidc_provider:?}");

    Ok(())
}
