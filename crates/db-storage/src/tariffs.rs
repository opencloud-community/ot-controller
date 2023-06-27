// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::schema::{external_tariffs, tariffs, users};
use crate::utils::Jsonb;
use chrono::{DateTime, Utc};
use core::fmt::Debug;
use database::{DbConnection, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use redis_args::{FromRedisValue, ToRedisArgs};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use types::{
    common::tariff::TariffResource,
    core::{TariffId, UserId},
};

types::diesel_newtype! {
    ExternalTariffId(String) => diesel::sql_types::Text
}

#[derive(
    Debug, Clone, Queryable, Identifiable, Serialize, Deserialize, ToRedisArgs, FromRedisValue,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct Tariff {
    pub id: TariffId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quotas: Jsonb<HashMap<String, u32>>,
    pub disabled_modules: Vec<String>,
    pub disabled_features: Vec<String>,
}

impl Tariff {
    pub async fn get(conn: &mut DbConnection, id: TariffId) -> Result<Self> {
        let tariff = tariffs::table
            .filter(tariffs::id.eq(id))
            .get_result(conn)
            .await?;

        Ok(tariff)
    }

    pub async fn get_all(conn: &mut DbConnection) -> Result<Vec<Self>> {
        let tariffs = tariffs::table.load(conn).await?;

        Ok(tariffs)
    }

    pub async fn get_by_name(conn: &mut DbConnection, name: &str) -> Result<Self> {
        let query = tariffs::table.filter(tariffs::name.eq(name));

        let tariff = query.get_result(conn).await?;

        Ok(tariff)
    }

    pub async fn delete_by_id(conn: &mut DbConnection, id: TariffId) -> Result<()> {
        let query = diesel::delete(tariffs::table).filter(tariffs::id.eq(id));
        query.execute(conn).await?;
        Ok(())
    }

    pub async fn get_by_external_id(
        conn: &mut DbConnection,
        id: &ExternalTariffId,
    ) -> Result<Self> {
        let query = external_tariffs::table
            .filter(external_tariffs::external_id.eq(id))
            .inner_join(tariffs::table)
            .select(tariffs::all_columns);

        let tariff = query.get_result(conn).await?;

        Ok(tariff)
    }

    pub async fn get_by_user_id(conn: &mut DbConnection, id: &UserId) -> Result<Self> {
        let query = users::table
            .filter(users::id.eq(id))
            .inner_join(tariffs::table)
            .select(tariffs::all_columns);

        let tariff = query.get_result(conn).await?;

        Ok(tariff)
    }

    pub fn to_tariff_resource<T1, T2>(
        &self,
        available_modules: impl IntoIterator<Item = T1>,
        disabled_features: impl IntoIterator<Item = T2>,
    ) -> TariffResource
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let disabled_modules: FxHashSet<String> =
            HashSet::from_iter(self.disabled_modules.iter().cloned());
        let available_modules: FxHashSet<String> =
            HashSet::from_iter(available_modules.into_iter().map(Into::into));

        let enabled_modules = available_modules
            .difference(&disabled_modules)
            .cloned()
            .collect();

        let disabled_features = self
            .disabled_features
            .iter()
            .cloned()
            .chain(disabled_features.into_iter().map(Into::into))
            .collect();

        TariffResource {
            id: self.id,
            name: self.name.clone(),
            quotas: self.quotas.0.clone(),
            enabled_modules,
            disabled_features,
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tariffs)]
pub struct NewTariff {
    pub name: String,
    pub quotas: Jsonb<HashMap<String, u32>>,
    pub disabled_modules: Vec<String>,
    pub disabled_features: Vec<String>,
}

impl NewTariff {
    pub async fn insert(self, conn: &mut DbConnection) -> Result<Tariff> {
        let query = self.insert_into(tariffs::table);
        let tariff = query.get_result(conn).await?;

        Ok(tariff)
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = tariffs)]
pub struct UpdateTariff {
    pub name: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub quotas: Option<Jsonb<HashMap<String, u32>>>,
    pub disabled_modules: Option<Vec<String>>,
    pub disabled_features: Option<Vec<String>>,
}

impl UpdateTariff {
    pub async fn apply(self, conn: &mut DbConnection, tariff_id: TariffId) -> Result<Tariff> {
        let query = diesel::update(tariffs::table.filter(tariffs::id.eq(tariff_id))).set(self);
        let tariff = query.get_result(conn).await?;
        Ok(tariff)
    }
}

#[derive(Debug, Clone, Insertable, Identifiable, Queryable)]
#[diesel(primary_key(external_id))]
pub struct ExternalTariff {
    pub external_id: ExternalTariffId,
    pub tariff_id: TariffId,
}

impl ExternalTariff {
    pub async fn get_all_for_tariff(
        conn: &mut DbConnection,
        tariff_id: TariffId,
    ) -> Result<Vec<ExternalTariffId>> {
        let query = external_tariffs::table
            .filter(external_tariffs::tariff_id.eq(tariff_id))
            .select(external_tariffs::external_id);
        let external_ids = query.load(conn).await?;

        Ok(external_ids)
    }

    pub async fn delete_all_for_tariff(conn: &mut DbConnection, tariff_id: TariffId) -> Result<()> {
        let query = diesel::delete(external_tariffs::table)
            .filter(external_tariffs::tariff_id.eq(tariff_id));
        query.execute(conn).await?;
        Ok(())
    }

    pub async fn delete_all_for_tariff_by_external_id(
        conn: &mut DbConnection,
        tariff_id: TariffId,
        external_ids: &[ExternalTariffId],
    ) -> Result<()> {
        let query = diesel::delete(external_tariffs::table).filter(
            external_tariffs::tariff_id
                .eq(tariff_id)
                .and(external_tariffs::external_id.eq_any(external_ids)),
        );
        query.execute(conn).await?;
        Ok(())
    }

    pub async fn insert(self, conn: &mut DbConnection) -> Result<()> {
        let query = self.insert_into(external_tariffs::table);
        query.execute(conn).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn tariff_to_tariff_resource() {
        let tariff = Tariff {
            id: TariffId::nil(),
            name: "test".into(),
            created_at: Default::default(),
            updated_at: Default::default(),
            quotas: Default::default(),
            disabled_modules: vec![
                "whiteboard".to_string(),
                "timer".to_string(),
                "media".to_string(),
                "polls".to_string(),
            ],
            disabled_features: vec!["call_in".to_string()],
        };
        let available_modules = vec!["chat", "media", "polls", "whiteboard", "timer"];

        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "test",
            "quotas": {},
            "enabled_modules": ["chat"],
            "disabled_features": ["call_in"],
        });

        let actual = serde_json::to_value(
            tariff.to_tariff_resource(available_modules, Vec::<String>::new()),
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
