// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling authentication.

mod bearer_token;
mod resumption_token;

pub use bearer_token::BearerToken;
pub use resumption_token::ResumptionToken;
