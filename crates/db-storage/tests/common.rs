// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use database::DbConnection;
use opentalk_db_storage::tariffs::Tariff;
use opentalk_db_storage::tenants::{get_or_create_tenant_by_oidc_id, OidcTenantId};
use opentalk_db_storage::users::{NewUser, User};
use types::core::TariffStatus;

pub async fn make_user(
    conn: &mut DbConnection,
    firstname: &str,
    lastname: &str,
    display_name: &str,
) -> User {
    let tenant = get_or_create_tenant_by_oidc_id(conn, &OidcTenantId::from("default".to_owned()))
        .await
        .unwrap();
    let tariff = Tariff::get_by_name(conn, "OpenTalkDefaultTariff")
        .await
        .unwrap();

    NewUser {
        email: format!(
            "{}.{}@example.org",
            firstname.to_lowercase(),
            lastname.to_lowercase()
        ),
        title: "".into(),
        firstname: firstname.into(),
        lastname: lastname.into(),
        id_token_exp: 0,
        display_name: display_name.into(),
        language: "".into(),
        oidc_sub: format!("{firstname}{lastname}"),
        phone: None,
        tenant_id: tenant.id,
        tariff_id: tariff.id,
        tariff_status: TariffStatus::Default,
    }
    .insert(conn)
    .await
    .unwrap()
}
