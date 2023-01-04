// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::schema::{external_tariffs, tariffs, users};
use crate::utils::Jsonb;
use chrono::{DateTime, Utc};
use controller_settings::{DEFAULT_NAMESPACE, NAMESPACE_SEPARATOR};
use core::fmt::Debug;
use database::{DbConnection, Result};
use derive_more::{AsRef, Display, From, FromStr, Into};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_newtype::DieselNewtype;
use redis_args::{FromRedisValue, ToRedisArgs};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use types::common::tariff::TariffModuleResource;
use types::{
    common::tariff::TariffResource,
    core::{TariffId, UserId},
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
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct ExternalTariffId(String);

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
    disabled_modules: Vec<Option<String>>,
    disabled_features: Vec<Option<String>>,
}

impl Tariff {
    pub fn disabled_modules(&self) -> HashSet<String> {
        self.disabled_modules.iter().flatten().cloned().collect()
    }

    pub fn disabled_features(&self) -> HashSet<String> {
        self.disabled_features.iter().flatten().cloned().collect()
    }

    pub fn is_feature_disabled(&self, feature: &str) -> bool {
        self.disabled_features.contains(&Some(feature.to_string()))
    }

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

    pub fn to_tariff_resource<T1, T2, T3>(
        &self,
        disabled_features: impl IntoIterator<Item = T1>,
        module_features: Vec<(T2, impl IntoIterator<Item = T3>)>,
    ) -> TariffResource
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
    {
        let disabled_modules: FxHashSet<String> = HashSet::from_iter(self.disabled_modules());

        let disabled_features: HashSet<_> = HashSet::from_iter(
            self.disabled_features()
                .into_iter()
                .chain(disabled_features.into_iter().map(Into::into)),
        );

        let mut enabled_module_names = HashSet::<String>::new();
        let mut modules = HashMap::<String, TariffModuleResource>::new();

        module_features
            .into_iter()
            .for_each(|(module_name, feature_name)| {
                let module_name = module_name.into();
                if !disabled_modules.contains(module_name.as_str()) {
                    let features = HashSet::from_iter(
                        feature_name.into_iter().map(Into::into).filter(|feature| {
                            !disabled_features.contains(
                                format!("{module_name}{NAMESPACE_SEPARATOR}{feature}").as_str(),
                            )
                        }),
                    );
                    let module_resource = TariffModuleResource { features };
                    // The list of enabled module names is deprecated and provided only for backwards compatibility.
                    // It is replaced by a list of modules including their features.
                    enabled_module_names.insert(module_name.clone());
                    modules.insert(module_name, module_resource);
                }
            });

        // The list of disabled feature names is deprecated and provided only for backwards compatibility. Also for backwards
        // compatibility, all features from the default namespace are listed twice in this deprecated field (with and without
        // the namespace prefix).
        let disabled_feature_names = disabled_features
            .iter()
            .flat_map(|item| {
                match item
                    .strip_prefix(format!("{DEFAULT_NAMESPACE}{NAMESPACE_SEPARATOR}").as_str())
                {
                    Some(stripped_item) => {
                        vec![item.to_owned(), stripped_item.to_owned()]
                    }
                    None => {
                        vec![item.to_owned()]
                    }
                }
                .into_iter()
            })
            .collect();

        // The 'enabled_modules' and 'disabled_features' fields are deprecated.
        TariffResource {
            id: self.id,
            name: self.name.clone(),
            quotas: self.quotas.0.clone(),
            enabled_modules: enabled_module_names,
            disabled_features: disabled_feature_names,
            modules,
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
                Some("whiteboard".to_string()),
                Some("timer".to_string()),
                Some("media".to_string()),
                Some("polls".to_string()),
            ],
            disabled_features: vec![Some("chat::chat_feature_1".to_string())],
        };

        let module_features = Vec::from([
            (
                "chat".to_owned(),
                HashSet::from(["chat_feature_1".to_owned(), "chat_feature_2".to_owned()]),
            ),
            ("media".to_owned(), HashSet::new()),
            ("polls".to_owned(), HashSet::new()),
            ("whiteboard".to_owned(), HashSet::new()),
            ("timer".to_owned(), HashSet::new()),
        ]);

        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "test",
            "quotas": {},
            "enabled_modules": ["chat"],
            "disabled_features": ["chat::chat_feature_1"],
            "modules": {
                "chat": {
                    "features": ["chat_feature_2"]
                },
            },
        });

        let actual =
            serde_json::to_value(tariff.to_tariff_resource(Vec::<String>::new(), module_features))
                .unwrap();

        assert_eq!(actual, expected);
    }
}
