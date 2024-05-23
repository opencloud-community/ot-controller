// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod error;
mod redis;

pub(crate) use error::SignalingStorageError;
pub(crate) use redis::{
    delete_resumption_token, get_resumption_token_data, get_ticket, refresh_resumption_token,
    set_resumption_token_data_if_not_exists, set_ticket_ex,
};
