// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{RedisConnection, SignalingModuleError, SignalingRoomId};

pub(crate) mod group;
pub(crate) mod init;
pub(crate) mod session;

/// Remove all redis keys related to this room & module
#[tracing::instrument(name = "cleanup_protocol", skip(redis_conn))]
pub(crate) async fn cleanup(
    redis_conn: &mut RedisConnection,
    room_id: SignalingRoomId,
) -> Result<(), SignalingModuleError> {
    init::del(redis_conn, room_id).await?;
    group::del(redis_conn, room_id).await?;

    Ok(())
}
