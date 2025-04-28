// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use opentalk_controller_service::{oidc::OpenIdConnectUserInfo, phone_numbers::parse_phone_number};
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::DbConnection;
use opentalk_db_storage::{
    groups::{insert_user_into_groups, remove_user_from_groups, Group},
    tariffs::Tariff,
    users::{UpdateUser, User},
};
use opentalk_types_common::{tariffs::TariffStatus, users::DisplayName};

use super::{build_info_display_name, LoginResult};

/// Called when the `POST /auth/login` endpoint received an id-token with a `sub`+`tenant_id` combination that maps to
/// an existing user. Resets the expiry time of the id-token for the user. Also updates all fields in the database that
/// have changed since the last login.
///
/// The parameter `groups` contains the groups the user should be a part of, according to the id-token. This function
/// removes the user from all groups that are not in the list and adds them to the groups in the list.
///
/// Returns the user and all groups the user was removed from and added to.
pub(super) async fn update_user(
    settings: &Settings,
    conn: &mut DbConnection,
    user: User,
    info: OpenIdConnectUserInfo,
    groups: Vec<Group>,
    tariff: Tariff,
    tariff_status: TariffStatus,
) -> Result<LoginResult, CaptureApiError> {
    // Enforce the auto-generated display name if display name editing is prohibited
    let enforced_display_name = if settings.endpoints.disallow_custom_display_name {
        Some(build_info_display_name(&info))
    } else {
        None
    };

    let changeset = create_changeset(
        settings,
        &user,
        &info,
        enforced_display_name.as_ref(),
        tariff,
        tariff_status,
    )?;

    let user = if changeset.is_empty() {
        user
    } else {
        changeset.apply(conn, user.id).await?
    };

    conn.transaction(|conn| {
        async move {
            let curr_groups = Group::get_all_for_user(conn, user.id).await?;

            // Add user to added groups
            let groups_added_to = difference_by(&groups, &curr_groups, |group| &group.id);
            if !groups_added_to.is_empty() {
                insert_user_into_groups(conn, &user, &groups_added_to).await?;
            }

            // Remove user from removed groups
            let groups_removed_from = difference_by(&curr_groups, &groups, |group| &group.id);
            if !groups_removed_from.is_empty() {
                remove_user_from_groups(conn, &user, &groups_removed_from).await?;
            }

            Ok(LoginResult::UserUpdated {
                user,
                groups_added_to,
                groups_removed_from,
            })
        }
        .scope_boxed()
    })
    .await
}

/// Create an [`UpdateUser`] changeset based on a comparison between `user` and `token_info`
fn create_changeset<'a>(
    settings: &Settings,
    user: &User,
    user_info: &'a OpenIdConnectUserInfo,
    enforced_display_name: Option<&'a DisplayName>,
    tariff: Tariff,
    tariff_status: TariffStatus,
) -> Result<UpdateUser<'a>, CaptureApiError> {
    let User {
        id: _,
        id_serial: _,
        oidc_sub: _,
        email,
        title: _,
        firstname,
        lastname,
        avatar_url,
        timezone,
        language: _,
        display_name,
        dashboard_theme: _,
        conference_theme: _,
        phone,
        tenant_id: _,
        tariff_id,
        tariff_status: tariff_status_db,
        disabled_since: _,
    } = user;

    let mut changeset = UpdateUser::default();

    if firstname != &user_info.firstname {
        changeset.firstname = Some(&user_info.firstname);
    }

    if lastname != &user_info.lastname {
        changeset.lastname = Some(&user_info.lastname)
    }

    if avatar_url != &user_info.avatar_url {
        changeset.avatar_url = Some(user_info.avatar_url.as_deref())
    }

    if timezone != &user_info.timezone {
        changeset.timezone = Some(user_info.timezone)
    }

    if let Some(enforced_display_name) = enforced_display_name {
        if display_name != enforced_display_name {
            changeset.display_name = Some(enforced_display_name)
        }
    }

    if email != &user_info.email {
        changeset.email = Some(&user_info.email);
    }

    let token_phone = if let Some((call_in, phone_number)) = settings
        .raw
        .call_in
        .as_ref()
        .zip(user_info.phone_number.as_deref())
    {
        parse_phone_number(phone_number, call_in.default_country_code)
            .map(|p| p.format().mode(phonenumber::Mode::E164).to_string())
    } else {
        None
    };

    if phone != &token_phone {
        changeset.phone = Some(token_phone)
    }

    if tariff_id != &tariff.id {
        changeset.tariff_id = Some(tariff.id)
    }

    if tariff_status != *tariff_status_db {
        changeset.tariff_status = Some(tariff_status);
    }

    Ok(changeset)
}

/// Returns all elements that are in `a` but no `b`
fn difference_by<T: Clone, C: PartialEq>(a: &[T], b: &[T], f: impl Fn(&T) -> &C) -> Vec<T> {
    a.iter()
        .filter(|a| !b.iter().any(|b| f(a) == f(b)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::difference_by;

    #[test]
    fn difference() {
        let set_a = ['a', 'b', 'c'];
        let set_b = ['b', 'c', 'd'];

        let difference = difference_by(&set_a, &set_b, |c| c);

        assert_eq!(difference, ['a']);
    }
}
