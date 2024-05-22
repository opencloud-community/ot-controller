// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod recording_storage;
mod redis;
mod volatile;

pub(crate) use recording_storage::RecordingStorage;
pub(super) use redis::{
    delete_all_streams, is_streaming_initialized, stream_exists, streams_contains_status,
    update_streams,
};
pub(crate) use redis::{get_stream, get_streams, set_stream, set_streams};
