// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{build_info_display_name, LoginResult};
use crate::api::util::parse_phone_number;
use crate::oidc::IdTokenInfo;
use controller_settings::Settings;
use database::DbConnection;
use db_storage::events::email_invites::EventEmailInvite;
use db_storage::groups::{insert_user_into_groups, Group};
use db_storage::tariffs::Tariff;
use db_storage::tenants::Tenant;
use db_storage::users::NewUser;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use types::core::TariffStatus;

/// Called when `POST /auth/login` receives an id-token with a new `sub` + `tenant_id` field combination. Creates a new
/// user in the given tenant using the information extracted from the id-token claims.
/// Also inserts the user in the previously created groups. (Group membership is also taken from the id-token.)
///
/// If any email-invites in the given tenant exist for the new user's email, they will be migrated to user-invites.
///
/// Returns the created user, their groups and all events they are invited to.
pub(super) async fn create_user(
    settings: &Settings,
    conn: &mut DbConnection,
    info: IdTokenInfo,
    tenant: Tenant,
    groups: Vec<Group>,
    tariff: Tariff,
    tariff_status: TariffStatus,
) -> database::Result<LoginResult> {
    let info_display_name = build_info_display_name(&info);

    let phone_number =
        if let Some((call_in, phone_number)) = settings.call_in.as_ref().zip(info.phone_number) {
            parse_phone_number(&phone_number, call_in.default_country_code)
                .map(|p| p.format().mode(phonenumber::Mode::E164).to_string())
        } else {
            None
        };

    conn.transaction(|conn| {
        async move {
            let user = NewUser {
                oidc_sub: info.sub,
                email: info.email,
                title: String::new(),
                display_name: info_display_name,
                firstname: info.firstname,
                lastname: info.lastname,
                id_token_exp: info.expiration.timestamp(),
                // TODO: try to get user language from accept-language header
                language: settings.defaults.user_language.clone(),
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
