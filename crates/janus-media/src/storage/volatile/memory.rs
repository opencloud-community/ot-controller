// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[derive(Debug, Clone, Default)]
pub(super) struct MemoryMediaState {}

impl MemoryMediaState {
    #[cfg(test)]
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}
