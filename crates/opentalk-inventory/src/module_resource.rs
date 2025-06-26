// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::module_resources::{Filter, ModuleResource, NewModuleResource, Operation};
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId};

use crate::Result;

/// A trait for retrieving and storing module resource entities.
#[async_trait::async_trait]
pub trait ModuleResourceInventory {
    /// Create a new module resource.
    async fn create_module_resource(
        &mut self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource>;

    /// Get one or multiple module resources.
    async fn get_module_resources(
        &mut self,
        resource_filter: Filter,
    ) -> Result<Vec<ModuleResource>>;

    /// Patch the contents of one or multiple module resources.
    async fn patch_module_resources(
        &mut self,
        resource_filter: Filter,
        operations: Vec<Operation>,
    ) -> Result<Vec<ModuleResource>>;

    /// Get all module resources associated with a room.
    async fn get_all_module_ids_for_room(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<ModuleResourceId>>;

    /// Delete all module resources of a room.
    async fn delete_all_module_resources_for_room(&mut self, room_id: RoomId) -> Result<()>;
}
