// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling tariffs.

mod quota_type;
mod tariff_id;
mod tariff_status;

pub use quota_type::QuotaType;
pub use tariff_id::TariffId;
pub use tariff_status::{TariffStatus, TariffStatusType};
