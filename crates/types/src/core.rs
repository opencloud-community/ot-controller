// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are considered to be in the core of OpenTalk.
//!
//! All core types are simple types (e.g. newtypes of primitive or other simple types),
//! and typically used by other types in this crate.

mod asset_id;
mod bearer_token;
mod breakout_room_id;
mod call_in;
mod date_time_tz;
mod event_id;
mod event_invite_status;
mod group_id;
mod group_name;
mod invite_code_id;
mod invite_role;
mod module_resource_id;
mod participant_id;
mod participation_kind;
mod resumption_token;
mod room_id;
mod streaming_key;
mod streaming_kind;
mod streaming_target_id;
mod tariff_id;
mod tariff_status;
mod tenant_id;
mod ticket_token;
mod time_zone;
mod timestamp;
mod user_id;

pub use asset_id::AssetId;
pub use bearer_token::BearerToken;
pub use breakout_room_id::BreakoutRoomId;
pub use call_in::{CallInId, CallInPassword};
pub use date_time_tz::DateTimeTz;
pub use event_id::EventId;
pub use event_invite_status::{EventInviteStatus, EventInviteStatusType};
pub use group_id::GroupId;
pub use group_name::GroupName;
pub use invite_code_id::InviteCodeId;
pub use invite_role::{InviteRole, InviteRoleType};
pub use module_resource_id::ModuleResourceId;
pub use participant_id::ParticipantId;
pub use participation_kind::ParticipationKind;
pub use resumption_token::ResumptionToken;
pub use room_id::RoomId;
pub use streaming_key::StreamingKey;
pub use streaming_kind::{StreamingKind, StreamingKindType};
pub use streaming_target_id::StreamingTargetId;
pub use tariff_id::TariffId;
pub use tariff_status::{TariffStatus, TariffStatusType};
pub use tenant_id::TenantId;
pub use ticket_token::TicketToken;
pub use time_zone::TimeZone;
pub use timestamp::Timestamp;
pub use user_id::UserId;
