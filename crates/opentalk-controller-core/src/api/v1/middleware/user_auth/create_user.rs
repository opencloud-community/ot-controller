// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
use opentalk_controller_service::{oidc::OpenIdConnectUserInfo, phone_numbers::parse_phone_number};
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::DbConnection;
use opentalk_db_storage::{
    events::email_invites::EventEmailInvite,
    groups::{Group, insert_user_into_groups},
    tariffs::Tariff,
    tenants::Tenant,
    users::NewUser,
};
use opentalk_types_common::{
    tariffs::TariffStatus,
    users::{Language, UserTitle},
};

use super::{LoginResult, build_info_display_name};

/// Called when `POST /auth/login` receives an id-token with a new `sub` + `tenant_id` field combination. Creates a new
/// user in the given tenant using the information extracted from the id-token claims.
/// Also inserts the user in the previously created groups. (Group membership is also taken from the id-token.)
///
/// If any email-invites in the given tenant exist for the new user's email, they will be migrated to user-invites.
///
/// Returns the created user, their groups and all events they are invited to.
#[allow(clippy::too_many_arguments)]
pub(super) async fn create_user(
    settings: &Settings,
    conn: &mut DbConnection,
    info: OpenIdConnectUserInfo,
    tenant: &Tenant,
    groups: Vec<Group>,
    tariff: Tariff,
    tariff_status: TariffStatus,
    fallback_locale: Language,
) -> Result<LoginResult, CaptureApiError> {
    let info_display_name = build_info_display_name(&info);

    let phone_number = if let Some((call_in, phone_number)) =
        settings.call_in.as_ref().zip(info.phone_number.as_deref())
    {
        parse_phone_number(phone_number, call_in.default_country_code)
            .map(|p| p.format().mode(phonenumber::Mode::E164).to_string())
    } else {
        None
    };

    let language = info.locale.unwrap_or(fallback_locale);

    conn.transaction(|conn| {
        async move {
            let user = NewUser {
                oidc_sub: info.sub,
                email: info.email,
                title: UserTitle::new(),
                display_name: info_display_name,
                firstname: info.firstname,
                lastname: info.lastname,
                avatar_url: info.avatar_url,
                language,
                phone: phone_number,
                tenant_id: tenant.id,
                tariff_id: tariff.id,
                tariff_status,
            }
            .insert(conn)
            .await?;

            insert_user_into_groups(conn, &user, &groups).await?;

            let event_and_room_ids = EventEmailInvite::migrate_to_user_invites(conn, &user).await?;

            Ok(LoginResult::UserCreated {
                user,
                groups,
                event_and_room_ids,
            })
        }
        .scope_boxed()
    })
    .await
}
