// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    delete,
    web::{Data, Path, ReqData},
};
use kustos::prelude::*;
use opentalk_controller_utils::deletion::{Deleter as _, RoomDeleter};
use opentalk_database::Db;
use opentalk_db_storage::users::User;
use opentalk_signaling_core::{ExchangeHandle, ObjectStorage};
use opentalk_types::{api::error::ApiError, core::RoomId};

use crate::{api::internal::NoContent, settings::SharedSettingsActix};

/// API Endpoint *DELETE /rooms/{room_id}*
///
/// Deletes the room and owned resources and linked events. This endpoint is rather complex as it
/// deletes multiple underlying REST exposed resources.
/// We need to check if we have access to all resources that need to be removed during this operation, and
/// we need to make sure to delete all related authz permissions of those resources.
///
/// We cannot rely on DB cascading as this would result in idling permissions.
///
/// Important:
/// Access checks should not be handled via a middleware but instead done inside, as this deletes multiple resources
#[delete("/rooms/{room_id}")]
pub async fn delete(
    settings: SharedSettingsActix,
    db: Data<Db>,
    storage: Data<ObjectStorage>,
    exchange_handle: Data<ExchangeHandle>,
    room_id: Path<RoomId>,
    current_user: ReqData<User>,
    authz: Data<Authz>,
) -> Result<NoContent, ApiError> {
    let room_id = room_id.into_inner();
    let current_user = current_user.into_inner();
    let settings = settings.load_full();

    let mut conn = db.get_conn().await?;

    let deleter = RoomDeleter::new(room_id, false);
    deleter
        .perform(
            log::logger(),
            &mut conn,
            &authz,
            Some(current_user.id),
            exchange_handle.as_ref().clone(),
            &settings,
            &storage,
        )
        .await?;

    Ok(NoContent {})
}
