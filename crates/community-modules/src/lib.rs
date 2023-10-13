// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use async_trait::async_trait;
use chat::Chat;
use integration::Integration;
use janus_media::Media;
use opentalk_core::Core;
use polls::Polls;
use protocol::Protocol;
use shared_folder::SharedFolder;
use signaling_core::{ModulesRegistrar, RegisterModules};
use timer::Timer;
use whiteboard::Whiteboard;

pub struct CommunityModules;

#[async_trait(?Send)]
impl RegisterModules for CommunityModules {
    async fn register(registrar: &mut impl ModulesRegistrar) -> Result<()> {
        registrar.register::<Core>().await?;
        registrar.register::<Chat>().await?;
        registrar.register::<Integration>().await?;
        registrar.register::<Media>().await?;
        registrar.register::<Polls>().await?;
        registrar.register::<Protocol>().await?;
        registrar.register::<SharedFolder>().await?;
        registrar.register::<Timer>().await?;
        registrar.register::<Whiteboard>().await
    }
}
