// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Facade library for accessing the OpenTalk data inventory.
//!
//! The production inventory implementation will usually be backed by a database
//! This abstraction helps to maintain proper separation of concerns, and to
//! keep the usage of it testable using e.g. mocks.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

mod asset;
pub mod error;
mod event;
mod event_invite;
mod event_shared_folder;
mod event_training_participation_report;
mod group;
mod inventory;
mod inventory_provider;
mod job_execution;
mod module_resource;
mod room;
mod room_invite;
mod room_sip_config;
mod room_streaming_target;
mod tariff;
mod tenant;
mod transaction;
mod transaction_manager;
mod user;

pub mod utils;

pub use asset::AssetInventory;
pub use error::Error;
pub use event::EventInventory;
pub use event_invite::EventInviteInventory;
pub use event_shared_folder::EventSharedFolderInventory;
pub use event_training_participation_report::EventTrainingParticipationReportInventory;
pub use group::GroupInventory;
pub use inventory::Inventory;
pub use inventory_provider::InventoryProvider;
pub use job_execution::JobExecutionInventory;
pub use module_resource::ModuleResourceInventory;
pub use room::RoomInventory;
pub use room_invite::RoomInviteInventory;
pub use room_sip_config::RoomSipConfigInventory;
pub use room_streaming_target::RoomStreamingTargetInventory;
pub use tariff::TariffInventory;
pub use tenant::TenantInventory;
pub use transaction::transaction;
pub use transaction_manager::TransactionManager;
pub use user::UserInventory;

/// The result type typically used for functions in this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
