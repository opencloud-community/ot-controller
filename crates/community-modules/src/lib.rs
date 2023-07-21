// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use chat::Chat;
use controller::Controller;
use core::Core;
use janus_media::Media;
use polls::Polls;
use protocol::Protocol;
use shared_folder::SharedFolder;
use timer::Timer;
use whiteboard::Whiteboard;

pub async fn register(controller: &mut Controller) -> Result<()> {
    controller.register::<Core>().await?;
    controller.register::<Chat>().await?;
    controller.register::<Media>().await?;
    controller.register::<Polls>().await?;
    controller.register::<Protocol>().await?;
    controller.register::<SharedFolder>().await?;
    controller.register::<Timer>().await?;
    controller.register::<Whiteboard>().await?;
    Ok(())
}
