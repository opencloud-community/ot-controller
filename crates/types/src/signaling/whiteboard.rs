// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `whiteboard` namespace

pub mod command;
pub mod event;
pub mod state;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "whiteboard";
