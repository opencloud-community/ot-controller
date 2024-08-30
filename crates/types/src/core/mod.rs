// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are considered to be in the core of OpenTalk.
//!
//! All core types are simple types (e.g. newtypes of primitive or other simple types),
//! and typically used by other types in this crate.

mod streaming_kind;
mod streaming_target_id;
mod tariff_id;
mod tariff_status;
mod tenant_id;
mod ticket_token;
mod timestamp;
mod user_id;

pub use streaming_kind::{StreamingKind, StreamingKindType};
pub use streaming_target_id::StreamingTargetId;
pub use tariff_id::TariffId;
pub use tariff_status::{TariffStatus, TariffStatusType};
pub use tenant_id::TenantId;
pub use ticket_token::TicketToken;
pub use timestamp::Timestamp;
pub use user_id::UserId;
