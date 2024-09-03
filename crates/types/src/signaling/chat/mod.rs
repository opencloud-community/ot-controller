// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `chat` namespace

pub mod peer_state;
pub mod state;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "chat";
