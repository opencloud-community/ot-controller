// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;
use opentalk_signaling_core::{ModulesRegistrar, RegisterModules};
use opentalk_signaling_module_automod::Automod;
use opentalk_signaling_module_chat::Chat;
use opentalk_signaling_module_core::Core;
use opentalk_signaling_module_integration::Integration;
use opentalk_signaling_module_legal_vote::LegalVote;
use opentalk_signaling_module_livekit::Livekit;
use opentalk_signaling_module_meeting_notes::MeetingNotes;
use opentalk_signaling_module_meeting_report::MeetingReport;
use opentalk_signaling_module_polls::Polls;
use opentalk_signaling_module_shared_folder::SharedFolder;
use opentalk_signaling_module_subroom_audio::SubroomAudio;
use opentalk_signaling_module_timer::Timer;
use opentalk_signaling_module_training_participation_report::TrainingParticipationReport;
use opentalk_signaling_module_whiteboard::Whiteboard;

pub struct CommunityModules;

#[async_trait(?Send)]
impl RegisterModules for CommunityModules {
    async fn register<E>(registrar: &mut impl ModulesRegistrar<Error = E>) -> Result<(), E> {
        registrar.register::<Core>().await?;
        registrar.register::<Chat>().await?;
        registrar.register::<LegalVote>().await?;
        registrar.register::<Automod>().await?;
        registrar.register::<Integration>().await?;
        registrar.register::<Livekit>().await?;
        registrar.register::<Polls>().await?;
        registrar.register::<MeetingNotes>().await?;
        registrar.register::<SharedFolder>().await?;
        registrar.register::<Timer>().await?;
        registrar.register::<Whiteboard>().await?;
        registrar.register::<MeetingReport>().await?;
        registrar.register::<SubroomAudio>().await?;
        registrar.register::<TrainingParticipationReport>().await
    }
}
