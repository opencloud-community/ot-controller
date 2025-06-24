// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::module_resources::{Filter, ModuleResource, NewModuleResource, Operation};
use opentalk_inventory::{
    error::{JsonOperationSnafu, StorageBackendSnafu},
    ModuleResourceInventory,
};
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl ModuleResourceInventory for DatabaseConnection {
    #[tracing::instrument(err, skip_all)]
    async fn create_module_resource(
        &mut self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource> {
        resource
            .insert(&mut self.inner)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_module_resources(
        &mut self,
        resource_filter: Filter,
    ) -> Result<Vec<ModuleResource>> {
        ModuleResource::get(&mut self.inner, resource_filter)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn patch_module_resources(
        &mut self,
        resource_filter: Filter,
        operations: Vec<Operation>,
    ) -> Result<Vec<ModuleResource>> {
        ModuleResource::patch(&mut self.inner, resource_filter, operations)
            .await
            .context(JsonOperationSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_all_module_ids_for_room(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<ModuleResourceId>> {
        ModuleResource::get_all_ids_for_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }

    #[tracing::instrument(err, skip_all)]
    async fn delete_all_module_resources_for_room(&mut self, room_id: RoomId) -> Result<()> {
        ModuleResource::delete_by_room(&mut self.inner, room_id)
            .await
            .context(StorageBackendSnafu)
    }
}
