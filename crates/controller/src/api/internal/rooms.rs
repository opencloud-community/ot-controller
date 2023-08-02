// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::api::internal::NoContent;
use crate::api::v1::response::ApiError;
use crate::settings::SharedSettingsActix;
use actix_web::delete;
use actix_web::web::{Data, Path, ReqData};
use controller_utils::deletion::{Deleter as _, Error, RoomDeleter};
use database::Db;
use db_storage::users::User;
use kustos::prelude::*;
use signaling_core::{ExchangeHandle, ObjectStorage};
use types::core::RoomId;

impl From<Error> for ApiError {
    fn from(value: Error) -> Self {
        match value {
            Error::Database(e) => ApiError::from(e),
            Error::Kustos(e) => ApiError::from(e),
            Error::Forbidden => ApiError::forbidden(),
            Error::ObjectDeletion(e) => ApiError::from(e),
            Error::SharedFoldersNotConfigured => {
                ApiError::bad_request().with_message("No shared folder configured for this server")
            }
            Error::NextcloudClient(_e) => {
                ApiError::internal().with_message("Error performing actions on the NextCloud")
            }
            Error::Custom(e) => ApiError::internal().with_message(e.to_string()),
        }
    }
}

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
