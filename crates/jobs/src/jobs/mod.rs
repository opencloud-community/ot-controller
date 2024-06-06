// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Jobs that can be run in the OpenTalk job execution system

mod adhoc_event_cleanup;
mod event_cleanup;
mod invite_cleanup;
mod keycloak_account_sync;
mod room_cleanup;
mod self_check;
mod sync_storage_files;
mod user_cleanup;

pub use adhoc_event_cleanup::AdhocEventCleanup;
pub use event_cleanup::EventCleanup;
pub use invite_cleanup::InviteCleanup;
pub use keycloak_account_sync::KeycloakAccountSync;
pub use room_cleanup::RoomCleanup;
pub use self_check::SelfCheck;
pub use sync_storage_files::SyncStorageFiles;
pub use user_cleanup::UserCleanup;
