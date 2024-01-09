// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod access;
pub mod error;
pub mod resource;
pub mod subject;

#[cfg(feature = "internal")]
pub mod internal;
