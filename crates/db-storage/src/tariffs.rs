// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;
use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use derive_more::{AsRef, Display, From, FromStr, Into};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DbConnection, Result};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_types::common::tariff::{TariffModuleResource, TariffResource};
use opentalk_types_common::{
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    tariffs::{QuotaType, TariffId},
    users::UserId,
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{
    schema::{external_tariffs, tariffs, users},
    utils::Jsonb,
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
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Serialize,
    Deserialize,
    ToRedisArgs,
    FromRedisValue,
    PartialEq,
    Eq,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct Tariff {
    pub id: TariffId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quotas: Jsonb<BTreeMap<QuotaType, u64>>,
    pub disabled_modules: Vec<Option<ModuleId>>,
    pub disabled_features: Vec<Option<ModuleFeatureId>>,
}

impl Tariff {
    pub fn quota(&self, quota: &QuotaType) -> Option<u64> {
        self.quotas.0.get(quota).copied()
    }

    pub fn disabled_modules(&self) -> BTreeSet<ModuleId> {
        self.disabled_modules.iter().flatten().cloned().collect()
    }

    pub fn disabled_features(&self) -> BTreeSet<ModuleFeatureId> {
        self.disabled_features.iter().flatten().cloned().collect()
    }

    pub fn is_feature_disabled(&self, module_feature: &ModuleFeatureId) -> bool {
        self.disabled_features
            .contains(&Some(module_feature.clone()))
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

    pub fn to_tariff_resource(
        &self,
        disabled_features: impl IntoIterator<Item = ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, impl IntoIterator<Item = FeatureId>>,
    ) -> TariffResource {
        let disabled_modules = self.disabled_modules();

        let disabled_features: BTreeSet<_> = BTreeSet::from_iter(
            self.disabled_features()
                .into_iter()
                .chain(disabled_features.into_iter().map(Into::into)),
        );

        let mut enabled_modules = BTreeSet::<ModuleId>::new();
        let mut modules = BTreeMap::<ModuleId, TariffModuleResource>::new();

        module_features
            .into_iter()
            .for_each(|(module_id, feature_id)| {
                if !disabled_modules.contains(&module_id) {
                    let features: BTreeSet<FeatureId> =
                        BTreeSet::from_iter(feature_id.into_iter().filter(|feature| {
                            !disabled_features.contains(&ModuleFeatureId {
                                module: module_id.clone(),
                                feature: feature.clone(),
                            })
                        }));
                    let module_resource = TariffModuleResource { features };
                    // The list of enabled module names is deprecated and provided only for backwards compatibility.
                    // It is replaced by a list of modules including their features.
                    enabled_modules.insert(module_id.clone());
                    modules.insert(module_id, module_resource);
                }
            });

        // The 'enabled_modules' and 'disabled_features' fields are deprecated.
        TariffResource {
            id: self.id,
            name: self.name.clone(),
            quotas: self.quotas.0.clone(),
            enabled_modules,
            disabled_features,
            modules,
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = tariffs)]
pub struct NewTariff {
    pub name: String,
    pub quotas: Jsonb<BTreeMap<QuotaType, u64>>,
    pub disabled_modules: Vec<ModuleId>,
    pub disabled_features: Vec<ModuleFeatureId>,
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
    pub quotas: Option<Jsonb<BTreeMap<QuotaType, u64>>>,
    pub disabled_modules: Option<Vec<ModuleId>>,
    pub disabled_features: Option<Vec<ModuleFeatureId>>,
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn tariff_to_tariff_resource() {
        let tariff = Tariff {
            id: TariffId::nil(),
            name: "test".into(),
            created_at: Default::default(),
            updated_at: Default::default(),
            quotas: Default::default(),
            disabled_modules: vec![
                Some("whiteboard".parse().expect("valid module id")),
                Some("timer".parse().expect("valid module id")),
                Some("media".parse().expect("valid module id")),
                Some("polls".parse().expect("valid module id")),
            ],
            disabled_features: vec![Some(
                "chat::chat_feature_1"
                    .parse()
                    .expect("valid module feature id"),
            )],
        };

        let module_features = BTreeMap::from([
            (
                "chat".parse().expect("valid module id"),
                BTreeSet::from([
                    "chat_feature_1".parse().expect("valid feature id"),
                    "chat_feature_2".parse().expect("valid feature id"),
                ]),
            ),
            ("media".parse().expect("valid moudle id"), BTreeSet::new()),
            ("polls".parse().expect("valid module id"), BTreeSet::new()),
            (
                "whiteboard".parse().expect("valid module id"),
                BTreeSet::new(),
            ),
            ("timer".parse().expect("valid module id"), BTreeSet::new()),
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

        let actual = serde_json::to_value(
            tariff.to_tariff_resource(BTreeSet::<ModuleFeatureId>::new(), module_features),
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
