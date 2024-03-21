// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_chat::Chat;
use opentalk_core::Core;
use opentalk_integration::Integration;
use opentalk_janus_media::Media;
use opentalk_polls::Polls;
use opentalk_protocol::Protocol;
use opentalk_shared_folder::SharedFolder;
use opentalk_signaling_core::{ModulesRegistrar, RegisterModules};
use opentalk_timer::Timer;
use opentalk_whiteboard::Whiteboard;

pub struct CommunityModules;

#[async_trait(?Send)]
impl RegisterModules for CommunityModules {
    async fn register<E>(registrar: &mut impl ModulesRegistrar<Error = E>) -> Result<(), E> {
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
