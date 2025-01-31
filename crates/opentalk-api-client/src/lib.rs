// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod requests;

mod authorization;
mod client;
mod open_talk_api_client;

pub use authorization::InviteCodeAuthorization;
pub use client::Client;
pub use open_talk_api_client::OpenTalkApiClient;
