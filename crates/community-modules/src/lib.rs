// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_chat::Chat;
use opentalk_core::Core;
use opentalk_integration::Integration;
use opentalk_livekit::Livekit;
use opentalk_meeting_notes::MeetingNotes;
use opentalk_meeting_report::MeetingReport;
use opentalk_polls::Polls;
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
        registrar.register::<Livekit>().await?;
        registrar.register::<Polls>().await?;
        registrar.register::<MeetingNotes>().await?;
        registrar.register::<SharedFolder>().await?;
        registrar.register::<Timer>().await?;
        registrar.register::<Whiteboard>().await?;
        registrar.register::<MeetingReport>().await
    }
}
