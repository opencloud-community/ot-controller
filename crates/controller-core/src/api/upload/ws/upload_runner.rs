// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::Db;
use opentalk_signaling_core::{assets, ObjectStorage, ObjectStorageError};
use opentalk_types::core::RoomId;
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn run_upload(
    storage: Arc<ObjectStorage>,
    db: Arc<Db>,
    room_id: RoomId,
    filename: String,
    receiver_stream: UnboundedReceiverStream<Result<bytes::Bytes, ObjectStorageError>>,
) {
    let result = assets::save_asset(
        &storage,
        db,
        room_id,
        Some("recording"),
        filename,
        "recording-render",
        receiver_stream,
    )
    .await;

    if let Err(e) = result {
        log::error!("Error saving asset, {}", e);
    }
}
