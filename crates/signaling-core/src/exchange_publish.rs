// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[derive(Debug, Clone)]
pub struct ExchangePublish {
    pub routing_key: String,
    pub message: String,
}
