// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `protocl` namespace

pub mod command;
pub mod event;
pub mod peer_state;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "protocol";
