// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

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

impl RegisterModules for CommunityModules {
    fn register(registrar: &mut impl ModulesRegistrar) {
        registrar.register::<Core>();
        registrar.register::<Chat>();
        registrar.register::<Integration>();
        registrar.register::<Media>();
        registrar.register::<Polls>();
        registrar.register::<Protocol>();
        registrar.register::<SharedFolder>();
        registrar.register::<Timer>();
        registrar.register::<Whiteboard>();
    }
}
