// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains the user specific database structs amd queries
use super::groups::{Group, UserGroupRelation};
use super::schema::{groups, users};
use crate::{levenshtein, lower, soundex};
use controller_settings::Settings;
use database::{DbConnection, Paginate, Result};
use derive_more::{AsRef, Display, From, FromStr, Into};
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, GroupedBy, Identifiable, Insertable,
    OptionalExtension, QueryDsl, Queryable, TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use diesel_newtype::DieselNewtype;
use serde::{Deserialize, Serialize};
use std::fmt;
use types::{
    api::v1::users::{PrivateUserProfile, PublicUserProfile},
    core::{TariffId, TariffStatus, TenantId, UserId},
};

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct SerialUserId(i64);

const MAX_USER_SEARCH_RESULTS: usize = 50;

/// Diesel user struct
///
/// Is used as a result in various queries. Represents a user column
#[derive(Clone, Queryable, Identifiable, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub id_serial: SerialUserId,
    pub oidc_sub: String,
    pub email: String,
    pub title: String,
    pub firstname: String,
    pub lastname: String,
    pub id_token_exp: i64,
    pub language: String,
    pub display_name: String,
    pub dashboard_theme: String,
    pub conference_theme: String,
    pub phone: Option<String>,
    pub tenant_id: TenantId,
    pub tariff_id: TariffId,
    pub tariff_status: TariffStatus,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("first_name", &self.firstname)
            .field("last_name", &self.lastname)
            .finish()
    }
}

impl User {
    /// Get a user with the given `id`
    ///
    /// If no user exists with `user_id` this returns an Error
    #[tracing::instrument(err, skip_all)]
    pub async fn get(conn: &mut DbConnection, user_id: UserId) -> Result<User> {
        let user = users::table
            .filter(users::id.eq(user_id))
            .get_result(conn)
            .await?;

        Ok(user)
    }

    /// Get a user with the given `id` inside a tenant
    ///
    /// If no user exists with `user_id` this returns an Error
    #[tracing::instrument(err, skip_all)]
    pub async fn get_filtered_by_tenant(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<User> {
        let user = users::table
            .filter(users::id.eq(user_id).and(users::tenant_id.eq(tenant_id)))
            .get_result(conn)
            .await?;

        Ok(user)
    }

    /// Get a user with the given id
    ///
    /// Returns None if no user matches `email`
    #[tracing::instrument(err, skip_all)]
    pub async fn get_by_email(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        email: &str,
    ) -> Result<Option<User>> {
        let user = users::table
            .filter(users::tenant_id.eq(tenant_id).and(users::email.eq(email)))
            .get_result(conn)
            .await
            .optional()?;

        Ok(user)
    }

    /// Get one or more users with the given phone number
    #[tracing::instrument(err, skip_all)]
    pub async fn get_by_phone(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        phone: &str,
    ) -> Result<Vec<User>> {
        let users = users::table
            .filter(users::tenant_id.eq(tenant_id).and(users::phone.eq(phone)))
            .get_results(conn)
            .await?;

        Ok(users)
    }

    /// Get all users alongside their current groups
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_with_groups(conn: &mut DbConnection) -> Result<Vec<(User, Vec<Group>)>> {
        let users_query = users::table.order_by(users::id.desc());
        let users = users_query.load(conn).await?;

        let groups_query = UserGroupRelation::belonging_to(&users).inner_join(groups::table);
        let groups: Vec<Vec<(UserGroupRelation, Group)>> = groups_query
            .load::<(UserGroupRelation, Group)>(conn)
            .await?
            .grouped_by(&users);

        let users_with_groups = users
            .into_iter()
            .zip(groups)
            .map(|(user, groups)| (user, groups.into_iter().map(|(_, group)| group).collect()))
            .collect();

        Ok(users_with_groups)
    }

    /// Get all users paginated
    #[tracing::instrument(err, skip_all, fields(%limit, %page))]
    pub async fn get_all_paginated(
        conn: &mut DbConnection,
        limit: i64,
        page: i64,
    ) -> Result<(Vec<User>, i64)> {
        let query = users::table
            .order_by(users::id.desc())
            .paginate_by(limit, page);

        let users_with_total = query.load_and_count(conn).await?;

        Ok(users_with_total)
    }

    /// Get Users paginated and filtered by ids
    #[tracing::instrument(err, skip_all, fields(%limit, %page))]
    pub async fn get_by_ids_paginated(
        conn: &mut DbConnection,
        ids: &[UserId],
        limit: i64,
        page: i64,
    ) -> Result<(Vec<User>, i64)> {
        let query = users::table
            .filter(users::id.eq_any(ids))
            .order_by(users::id.desc())
            .paginate_by(limit, page);

        let users_with_total = query.load_and_count::<User, _>(conn).await?;

        Ok(users_with_total)
    }

    /// Returns all `User`s filtered by id
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_by_ids(conn: &mut DbConnection, ids: &[UserId]) -> Result<Vec<User>> {
        let query = users::table.filter(users::id.eq_any(ids));
        let users = query.load(conn).await?;

        Ok(users)
    }

    /// Get user with the given `sub` inside a tenant
    ///
    /// Returns None no user matched `sub`
    #[tracing::instrument(err, skip_all)]
    pub async fn get_by_oidc_sub(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        sub: &str,
    ) -> Result<Option<User>> {
        let user = users::table
            .filter(users::oidc_sub.eq(sub).and(users::tenant_id.eq(tenant_id)))
            .get_result(conn)
            .await
            .optional()?;

        Ok(user)
    }

    /// Get all users filtered by the given subs
    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_by_oidc_subs(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        subs: &[&str],
    ) -> Result<Vec<User>> {
        let users = users::table
            .filter(
                users::tenant_id
                    .eq(tenant_id)
                    .and(users::oidc_sub.eq_any(subs)),
            )
            .load(conn)
            .await?;

        Ok(users)
    }

    /// Find users by search string
    ///
    /// This looks for similarities of the search_str in the display_name, first+lastname and email
    #[tracing::instrument(err, skip_all)]
    pub async fn find(
        conn: &mut DbConnection,
        tenant_id: TenantId,
        search_str: &str,
        max_users: usize,
    ) -> Result<Vec<User>> {
        // IMPORTANT: lowercase it to match the index of the db and
        // remove all existing % in name and to avoid manipulation of the LIKE query.
        let search_str = search_str.replace('%', "").trim().to_lowercase();

        if search_str.is_empty() {
            return Ok(vec![]);
        }

        let like_query = format!("%{search_str}%");

        let lower_display_name = lower(users::display_name);

        let lower_first_lastname = lower(users::firstname.concat(" ").concat(users::lastname));

        let matches = users::table
            .filter(users::tenant_id.eq(tenant_id))
            .filter(
                // First try LIKE query on display_name
                lower_display_name.like(&like_query).or(
                    // Then try LIKE query with first+last name
                    lower_first_lastname
                        .like(&like_query)
                        // Then try LIKE query on email
                        .or(lower(users::email).like(&like_query))
                        //
                        // Then SOUNDEX on display_name
                        .or(soundex(lower_display_name)
                            .eq(soundex(&search_str))
                            // only take SOUNDEX results with a levenshtein score of lower than 5
                            .and(levenshtein(lower_display_name, &search_str).lt(5)))
                        //
                        // Then SOUNDEX on first+last name
                        .or(soundex(lower_first_lastname)
                            .eq(soundex(&search_str))
                            // only take SOUNDEX results with a levenshtein score of lower than 5
                            .and(levenshtein(lower_first_lastname, &search_str).lt(5))),
                ),
            )
            .order_by(levenshtein(lower_display_name, &search_str))
            .then_order_by(levenshtein(lower_first_lastname, &search_str))
            .then_order_by(users::id)
            .limit(MAX_USER_SEARCH_RESULTS.min(max_users) as i64)
            .load(conn)
            .await?;

        Ok(matches)
    }

    pub fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile {
        let avatar_url = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PublicUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            avatar_url,
        }
    }

    pub fn to_private_user_profile(&self, settings: &Settings) -> PrivateUserProfile {
        let avatar_url = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PrivateUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            dashboard_theme: self.dashboard_theme.clone(),
            conference_theme: self.conference_theme.clone(),
            avatar_url,
            language: self.language.clone(),
            tariff_status: self.tariff_status,
        }
    }
}

/// Helper function to turn an email address into libravatar URL.
pub fn email_to_libravatar_url(libravatar_url: &str, email: &str) -> String {
    format!("{}{:x}", libravatar_url, md5::compute(email))
}

/// Diesel insertable user struct
///
/// Represents fields that have to be provided on user insertion.
#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub oidc_sub: String,
    pub email: String,
    pub title: String,
    pub firstname: String,
    pub lastname: String,
    pub id_token_exp: i64,
    pub language: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub tenant_id: TenantId,
    pub tariff_id: TariffId,
    pub tariff_status: TariffStatus,
}

impl NewUser {
    pub async fn insert(self, conn: &mut DbConnection) -> Result<User> {
        let query = self.insert_into(users::table);
        let user = query.get_result(conn).await?;
        Ok(user)
    }
}

/// Diesel user struct for updates
///
/// Is used in update queries. None fields will be ignored on update queries
#[derive(Default, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub title: Option<&'a str>,
    pub email: Option<&'a str>,
    pub firstname: Option<&'a str>,
    pub lastname: Option<&'a str>,
    pub phone: Option<Option<String>>,
    pub display_name: Option<&'a str>,
    pub language: Option<&'a str>,
    pub id_token_exp: Option<i64>,
    pub dashboard_theme: Option<&'a str>,
    pub conference_theme: Option<&'a str>,
    // The tenant_id should never be updated!
    //pub tenant_id: Option<TenantId>,
    pub tariff_id: Option<TariffId>,
    pub tariff_status: Option<TariffStatus>,
}

impl UpdateUser<'_> {
    pub async fn apply(self, conn: &mut DbConnection, user_id: UserId) -> Result<User> {
        let query = diesel::update(users::table.filter(users::id.eq(user_id))).set(self);
        let user: User = query.get_result(conn).await?;
        Ok(user)
    }
}
