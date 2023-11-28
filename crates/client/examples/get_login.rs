// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use opentalk_client::{Client, OpenTalkApiClient};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const URL_ENV_VAR: &str = "OPENTALK_CONTROLLER_URL";
    let url = Url::parse(
        std::env::var(URL_ENV_VAR)
            .context(format!(
        "Please set the {URL_ENV_VAR} environment variable to the URL of the OpenTalk controller"
    ))?
            .as_str(),
    )?;
    let client = Client::new(url);

    let oidc_provider = client.get_login().await?;

    println!("OIDC provider: {oidc_provider:?}");

    Ok(())
}
