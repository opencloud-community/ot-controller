// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Spacedeck settings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SubroomAudio {
    /// Enable the subroom audio whisper functionality.
    pub enable_whisper: bool,
}

impl From<settings_file::SubroomAudio> for SubroomAudio {
    fn from(settings_file::SubroomAudio { enable_whisper }: settings_file::SubroomAudio) -> Self {
        Self { enable_whisper }
    }
}
