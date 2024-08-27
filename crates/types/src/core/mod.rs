// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are considered to be in the core of OpenTalk.
//!
//! All core types are simple types (e.g. newtypes of primitive or other simple types),
//! and typically used by other types in this crate.

mod file_extension;
mod group_id;
mod group_name;
mod invite_code_id;
mod invite_role;
mod module_resource_id;
pub mod one_or_many_btree_set;
pub mod one_or_many_vec;
mod participant_id;
mod participation_kind;
mod recurrence_pattern;
mod recurrence_rule;
mod resumption_token;
mod room_id;
mod room_password;
mod streaming_key;
mod streaming_kind;
mod streaming_target_id;
mod tariff_id;
mod tariff_status;
mod tenant_id;
mod ticket_token;
mod timestamp;
mod user_id;

pub use file_extension::{FileExtension, MAX_FILE_EXTENSION_LENGTH};
pub use group_id::GroupId;
pub use group_name::GroupName;
pub use invite_code_id::InviteCodeId;
pub use invite_role::{InviteRole, InviteRoleType};
pub use module_resource_id::ModuleResourceId;
#[cfg(feature = "serde")]
pub use one_or_many_btree_set::one_or_many_btree_set_option;
pub use one_or_many_btree_set::OneOrManyBTreeSet;
#[cfg(feature = "serde")]
pub use one_or_many_vec::one_or_many_vec_option;
pub use one_or_many_vec::OneOrManyVec;
pub use participant_id::ParticipantId;
pub use participation_kind::ParticipationKind;
pub use recurrence_pattern::{
    RecurrencePattern, TryFromRecurrenceRulesError, RECURRENCE_PATTERN_MAX_LEN,
};
pub use recurrence_rule::{ParseRecurrenceRuleError, RecurrenceRule, RECURRENCE_RULE_MAX_LEN};
pub use resumption_token::ResumptionToken;
pub use room_id::RoomId;
pub use room_password::{RoomPassword, MAX_ROOM_PASSWORD_LENGTH, MIN_ROOM_PASSWORD_LENGTH};
pub use streaming_key::StreamingKey;
pub use streaming_kind::{StreamingKind, StreamingKindType};
pub use streaming_target_id::StreamingTargetId;
pub use tariff_id::TariffId;
pub use tariff_status::{TariffStatus, TariffStatusType};
pub use tenant_id::TenantId;
pub use ticket_token::TicketToken;
pub use timestamp::Timestamp;
pub use user_id::UserId;
