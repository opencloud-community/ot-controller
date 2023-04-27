// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `media` namespace

mod trickle_candidate;

pub mod event;

pub use trickle_candidate::TrickleCandidate;
