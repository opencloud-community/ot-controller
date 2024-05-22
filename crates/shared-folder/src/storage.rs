// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod redis;

pub(crate) use redis::{
    delete_shared_folder, delete_shared_folder_initialized, get_shared_folder,
    is_shared_folder_initialized, set_shared_folder, set_shared_folder_initialized,
};
