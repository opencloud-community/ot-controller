// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

crate::diesel_newtype! {
    feature_gated:

    #[derive(Copy)] RoomId(uuid::Uuid) => diesel::sql_types::Uuid, "/rooms/"
}
