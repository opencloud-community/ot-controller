// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Utility to map a phone number to a users display name or convert it to a more readable format

use std::{convert::TryFrom, sync::Arc};

use opentalk_controller_service::phone_numbers::parse_phone_number;
use opentalk_controller_settings as settings;
use opentalk_database::Db;
use opentalk_db_storage::users::User;
use opentalk_types_common::{tenants::TenantId, users::DisplayName};
use phonenumber::PhoneNumber;

/// Try to map the provided phone number to a user
///
/// When the mapping fails or is disabled, the phone number may be formatted to the international phone format.
///
/// Returns the display name for a given SIP display name, e.g. a phone number
pub async fn display_name(
    db: &Arc<Db>,
    settings: &settings::CallIn,
    tenant_id: TenantId,
    phone_number_display_name: DisplayName,
) -> DisplayName {
    let parsed_number = if let Some(parsed_number) = parse_phone_number(
        phone_number_display_name.as_str(),
        settings.default_country_code,
    ) {
        parsed_number
    } else {
        // Failed to parse, return input
        return phone_number_display_name;
    };

    if settings.enable_phone_mapping {
        if let Some(display_name) =
            try_map_to_user_display_name(db, tenant_id, &parsed_number).await
        {
            return display_name;
        }
    }

    DisplayName::from_str_lossy(
        &parsed_number
            .format()
            .mode(phonenumber::Mode::International)
            .to_string(),
    )
}

/// Try to map the provided phone number to a user
///
/// The mapping fails if no user has the provided phone number configured or multiple
/// users have the provided phone number configured.
///
/// Returns [`None`] the phone number is invalid or cannot be parsed
async fn try_map_to_user_display_name(
    db: &Arc<Db>,
    tenant_id: TenantId,
    phone_number: &PhoneNumber,
) -> Option<DisplayName> {
    let phone_e164 = phone_number
        .format()
        .mode(phonenumber::Mode::E164)
        .to_string();

    let mut conn = db.get_conn().await.ok()?;
    let result = User::get_by_phone(&mut conn, tenant_id, &phone_e164).await;

    let users = match result {
        Ok(users) => users,
        Err(err) => {
            log::warn!(
                "Failed to get users by phone number from database {:?}",
                err
            );
            return None;
        }
    };

    if let Ok([user]) = <[_; 1]>::try_from(users) {
        Some(user.display_name)
    } else {
        None
    }
}
