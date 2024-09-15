// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `timer` namespace

mod started;
mod stop_kind;

pub use started::Started;
pub use stop_kind::StopKind;
