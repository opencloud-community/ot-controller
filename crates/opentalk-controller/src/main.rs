// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_core::Controller;
use opentalk_controller_service::Whatever;
use opentalk_signaling_modules::Modules;

#[actix_web::main]
async fn main() {
    opentalk_controller_core::try_or_exit(run()).await;
}

async fn run() -> Result<(), Whatever> {
    if let Some(controller) = Controller::create::<Modules>("OpenTalk Controller").await? {
        controller.run().await?;
    }

    Ok(())
}
