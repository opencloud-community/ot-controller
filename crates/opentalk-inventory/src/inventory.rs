// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    AssetInventory, EventInventory, EventInviteInventory, EventSharedFolderInventory,
    EventTrainingParticipationReportInventory, GroupInventory, JobExecutionInventory,
    ModuleResourceInventory, RoomInventory, RoomInviteInventory, RoomSipConfigInventory,
    RoomStreamingTargetInventory, TariffInventory, TenantInventory, TransactionManager,
    UserInventory,
};

/// A connection to the storage backend.
pub trait Inventory:
    Send
    + AssetInventory
    + EventInviteInventory
    + EventSharedFolderInventory
    + EventInventory
    + EventTrainingParticipationReportInventory
    + GroupInventory
    + JobExecutionInventory
    + ModuleResourceInventory
    + RoomInviteInventory
    + RoomSipConfigInventory
    + RoomInventory
    + RoomStreamingTargetInventory
    + TariffInventory
    + TenantInventory
    + UserInventory
    + TransactionManager
{
}
