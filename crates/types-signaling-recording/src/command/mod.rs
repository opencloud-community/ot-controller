// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

mod set_consent;
mod start_streaming;

pub use set_consent::SetConsent;
pub use start_streaming::StartStreaming;
