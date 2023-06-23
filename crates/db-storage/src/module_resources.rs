// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::schema::{module_resources, rooms};
use chrono::{DateTime, Utc};
use database::{DatabaseError, DbConnection, Result};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Jsonb;
use diesel::{ExpressionMethods, Identifiable, QueryDsl, Queryable};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;
use types::core::{ModuleResourceId, RoomId, TenantId, UserId};

#[derive(
    Debug, PartialEq, Serialize, strum::Display, strum::FromRepr, strum::AsRefStr, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
pub enum JsonPatchErrorCode {
    /// The given path is invalid
    #[strum(serialize = "ot_invalid_path", to_string = "invalid_path")]
    InvalidPath,

    /// Can only be thrown by the [`Test`](Operation::Test) when the compare returns `false`
    #[strum(serialize = "ot_value_not_equal", to_string = "value_not_equal")]
    ValueNotEqual,

    /// Can only be thrown by a [`Copy`](Operation::Copy) or [`Move`](Operation::Move) operation when the `from`
    /// parameter is invalid.
    #[strum(serialize = "ot_invalid_from_path", to_string = "invalid_from_path")]
    InvalidFromPath,
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("Json operation failed at index {at_index}: {error_code} - {details} ")]
pub struct JsonPatchError {
    pub error_code: JsonPatchErrorCode,
    /// Human readable error details
    pub details: String,
    /// The index of the of the operation that failed
    pub at_index: usize,
}

impl JsonPatchError {
    fn try_from_diesel_error(diesel_error: &diesel::result::Error) -> Option<Self> {
        if let diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            error_info,
        ) = diesel_error
        {
            let error_code = JsonPatchErrorCode::from_str(error_info.message()).ok()?;

            let details = error_info.details()?;

            let at_index = error_info.hint()?.parse().ok()?;

            return Some(JsonPatchError {
                error_code,
                details: details.into(),
                at_index,
            });
        }

        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsonOperationError {
    #[error("{0}")]
    JsonPatch(JsonPatchError),
    #[error("{0}")]
    Database(DatabaseError),
}

impl From<diesel::result::Error> for JsonOperationError {
    fn from(value: diesel::result::Error) -> Self {
        match JsonPatchError::try_from_diesel_error(&value) {
            Some(json_patch_error) => Self::JsonPatch(json_patch_error),
            None => Self::Database(value.into()),
        }
    }
}

/// Helper struct for filtering module resources
#[derive(Default)]
pub struct Filter {
    /// Filter by the UUID of the module resource
    id: Option<ModuleResourceId>,
    /// Filter by the namespace of the module resource
    namespace: Option<String>,
    /// Filter by the creator of the module resource
    created_by: Option<UserId>,
    /// Filter by the tag of the module resource
    tag: Option<String>,
    /// Filter by the content of the module resource
    json: Option<serde_json::Value>,
}

type ResourceFilter =
    Box<dyn BoxableExpression<module_resources::table, Pg, SqlType = diesel::sql_types::Bool>>;

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: ModuleResourceId) -> Self {
        self.id = Some(id);

        self
    }

    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);

        self
    }

    pub fn with_created_by(mut self, created_by: UserId) -> Self {
        self.created_by = Some(created_by);

        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);

        self
    }

    pub fn with_json(mut self, json: serde_json::Value) -> Self {
        self.json = Some(json);

        self
    }

    fn into_diesel_filter(self) -> Option<ResourceFilter> {
        fn append(query: &mut Option<ResourceFilter>, filter: ResourceFilter) {
            if let Some(query_) = query.take() {
                *query = Some(Box::new(query_.and(filter)));
            } else {
                *query = Some(filter);
            }
        }

        let mut query: Option<ResourceFilter> = None;

        if let Some(id) = self.id {
            append(&mut query, Box::new(module_resources::id.eq(id)));
        }

        if let Some(namespace) = self.namespace {
            append(
                &mut query,
                Box::new(module_resources::namespace.eq(namespace)),
            );
        }

        if let Some(created_by) = self.created_by {
            append(
                &mut query,
                Box::new(module_resources::created_by.eq(created_by)),
            );
        }

        if let Some(tag) = self.tag {
            // Unwrapping the Nullable<Bool> here with an if is_not_null then compare, else false
            append(
                &mut query,
                Box::new(
                    module_resources::tag
                        .is_not_null()
                        .and(module_resources::tag.eq(tag))
                        .assume_not_null(),
                ),
            );
        }

        if let Some(json) = self.json {
            append(&mut query, Box::new(module_resources::data.contains(json)));
        }

        query
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Queryable, Identifiable, Insertable)]
pub struct ModuleResource {
    pub id: ModuleResourceId,
    pub tenant_id: TenantId,
    pub room_id: RoomId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub namespace: String,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

impl ModuleResource {
    #[tracing::instrument(err, skip_all)]
    pub async fn get(conn: &mut DbConnection, filter: Filter) -> Result<Vec<Self>> {
        let filter = filter.into_diesel_filter().ok_or(DatabaseError::Custom(
            "Missing filter for get module_resource query".into(),
        ))?;

        let query = module_resources::table.filter(filter);

        let module_resources = query.get_results(conn).await?;

        Ok(module_resources)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_ids_for_room(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Vec<ModuleResourceId>> {
        let query = module_resources::table
            .select(module_resources::id)
            .filter(module_resources::room_id.eq(room_id));

        let module_resource_ids = query.load(conn).await?;

        Ok(module_resource_ids)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn get_all_with_creator_and_owner(
        conn: &mut DbConnection,
    ) -> Result<Vec<(ModuleResourceId, UserId, UserId)>> {
        let query = module_resources::table.inner_join(rooms::table).select((
            module_resources::id,
            module_resources::created_by,
            rooms::created_by,
        ));

        let module_resources_with_creator = query.load(conn).await?;

        Ok(module_resources_with_creator)
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete(conn: &mut DbConnection, filter: Filter) -> Result<()> {
        let filter = filter.into_diesel_filter().ok_or(DatabaseError::Custom(
            "Missing filter for delete query".into(),
        ))?;

        let query = diesel::delete(module_resources::table).filter(filter);

        query.execute(conn).await?;

        Ok(())
    }

    #[tracing::instrument(err, skip_all)]
    pub async fn delete_by_room(conn: &mut DbConnection, room_id: RoomId) -> Result<()> {
        diesel::delete(module_resources::table)
            .filter(module_resources::room_id.eq(room_id))
            .execute(conn)
            .await?;

        Ok(())
    }

    /// Update the targeted json data by a set of json patch [Operations](Operation)
    #[tracing::instrument(err, skip_all)]
    pub async fn patch(
        conn: &mut DbConnection,
        filter: Filter,
        operations: Vec<Operation>,
    ) -> Result<ModuleResource, JsonOperationError> {
        let filter = filter
            .into_diesel_filter()
            .ok_or(JsonOperationError::Database(DatabaseError::Custom(
                "Missing filter for patch query".into(),
            )))?;

        let query = diesel::update(module_resources::table).filter(filter);

        let query = query.set(module_resources::data.eq(ot_patch_json(
            module_resources::data,
            serde_json::to_value(operations).unwrap(),
        )));

        let module_resource = query.get_result(conn).await?;

        Ok(module_resource)
    }
}

/// Possible json patch operations based on [RFC6902](https://www.rfc-editor.org/rfc/rfc6902#section-4)
#[derive(Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    Add { path: String, value: Value },

    Remove { path: String },

    Replace { path: String, value: Value },

    Move { from: String, path: String },

    Copy { from: String, path: String },

    Test { path: String, value: Value },
}

sql_function! {
    fn ot_patch_json(target: Jsonb, changeset: Jsonb) -> Jsonb;
}

#[derive(Debug, Insertable)]
#[diesel(table_name = module_resources)]
pub struct NewModuleResource {
    pub tenant_id: TenantId,
    pub room_id: RoomId,
    pub created_by: UserId,
    pub namespace: String,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

impl NewModuleResource {
    #[tracing::instrument(err, skip_all)]
    pub async fn insert(self, conn: &mut DbConnection) -> Result<ModuleResource> {
        let module: ModuleResource = diesel::insert_into(module_resources::table)
            .values(self)
            .get_result(conn)
            .await?;

        Ok(module)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tenants::get_or_create_tenant_by_oidc_id;
    use crate::tenants::OidcTenantId;
    use serde_json::json;
    use serde_json::Value;
    use serial_test::serial;
    use test_util::assert_eq;
    use test_util::*;

    async fn init_resource(json: Value) -> (ModuleResourceId, DbConnection) {
        let db_ctx = test_util::database::DatabaseContext::new(false).await;

        let mut db_conn = db_ctx.db.get_conn().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        let room = db_ctx
            .create_test_room(RoomId::from_u128(0), user.id, false)
            .await
            .unwrap();

        let tenant = get_or_create_tenant_by_oidc_id(
            &mut db_conn,
            &OidcTenantId::from("OpenTalkDefaultTenant".to_owned()),
        )
        .await
        .unwrap();

        let m = NewModuleResource {
            tenant_id: tenant.id,
            created_by: user.id,
            room_id: room.id,
            namespace: "test".into(),
            tag: None,
            data: json,
        };

        let resource = m.insert(&mut db_conn).await.unwrap();

        (resource.id, db_conn)
    }

    async fn init_empty_resource() -> (ModuleResourceId, DbConnection) {
        init_resource(json!({})).await
    }

    #[actix_rt::test]
    #[serial]
    async fn add_single() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![Operation::Add {
            path: "/foo".into(),
            value: Value::String("bar".into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": "bar"
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn empty_path() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![Operation::Add {
            path: "".into(),
            value: Value::String("bar".into()),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidPath, err.error_code)
            }
            unexpected => panic!("Expected invalid_path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn add_invalid() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![Operation::Add {
            path: "/foo/bar/baz".into(),
            value: Value::String("bar".into()),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidPath, err.error_code)
            }
            unexpected => panic!("Expected invalid_path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn add_a_lot() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let mut json_compare = json!({});

        let mut operations = vec![];

        for i in 0..100 {
            let field = format!("foo{}", i);

            operations.push(Operation::Add {
                path: format!("/{}", field),
                value: Value::String("bar".into()),
            });

            if let Value::Object(hm) = &mut json_compare {
                hm.insert(field, "bar".into());
            }
        }

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data, json_compare);
    }

    #[actix_rt::test]
    #[serial]
    async fn add_single_empty() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![Operation::Add {
            path: "/".into(),
            value: Value::String("bar".into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data, "bar");
    }

    #[actix_rt::test]
    #[serial]
    async fn add_nested() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![
            Operation::Add {
                path: "/foo".into(),
                value: json!({"bar": 1}),
            },
            Operation::Add {
                path: "/foo/bar".into(),
                value: Value::Number(42.into()),
            },
        ];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": {
                "bar": 42
            }
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn add_to_array() {
        let initial_json = json!({
            "foo": ["a", "c"],
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Add {
            path: "/foo/1".into(),
            value: Value::String("b".into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": ["a", "b", "c"]
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn add_array() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![Operation::Add {
            path: "/foo".into(),
            value: Value::Array(vec![1.into(), 2.into(), 3.into()]),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": [1, 2 ,3]
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn add_multi() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![
            Operation::Add {
                path: "/foo".into(),
                value: Value::Number(1.into()),
            },
            Operation::Add {
                path: "/bar".into(),
                value: Value::Number(2.into()),
            },
        ];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": 1,
            "bar": 2,
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn update_multi_add_with_update() {
        let (_id, mut db_conn) = init_empty_resource().await;

        let operations = vec![
            Operation::Add {
                path: "/foo".into(),
                value: Value::Number(1.into()),
            },
            Operation::Add {
                path: "/bar".into(),
                value: Value::Number(2.into()),
            },
            Operation::Add {
                path: "/foo".into(),
                value: Value::Number(42.into()),
            },
        ];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": 42,
            "bar": 2,
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_remove() {
        let initial_json = json!({
            "foo": 1,
            "bar": 2
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Remove {
            path: "/foo".into(),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "bar": 2
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn remove_by_index() {
        let initial_json = json!({
            "foo": ["a", "b", "c"],
            "bar": {
                "0": "baz",
                "qux": "biz"
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![
            Operation::Remove {
                path: "/foo/0".into(),
            },
            Operation::Remove {
                path: "/bar/0".into(),
            },
        ];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": ["b", "c"],
            "bar": {
                "qux": "biz"
            },
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn remove_and_add_with_numbers() {
        let initial_json = json!({
            "foo": ["a", "b", "c"],
            "bar": {
                "0": "baz",
                "qux": "biz"
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![
            Operation::Remove {
                path: "/foo/0".into(),
            },
            Operation::Remove {
                path: "/bar/0".into(),
            },
            Operation::Add {
                path: "/bar/0".into(),
                value: Value::Number(42.into()),
            },
        ];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": ["b", "c"],
            "bar": {
                "0": 42,
                "qux": "biz"
            },
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_move() {
        let initial_json = json!({
            "foo": { "qux": 1 },
            "bar": {
                "baz": 2,
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Move {
            from: "/foo/qux".into(),
            path: "/bar/qux".into(),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": { },
            "bar": {
                "baz": 2,
                "qux": 1
            },
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn move_empty() {
        let initial_json = json!({
            "foo": { "qux": 1 },
            "bar": {
                "baz": 2,
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Move {
            from: "/".into(),
            path: "/bar/qux".into(),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidFromPath, err.error_code)
            }
            unexpected => panic!("Expected invalid_from_path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn move_into_child() {
        let initial_json = json!({
            "foo": {
                "bar": 42
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let from_path: String = "/foo".into();
        let to_path: String = "/foo/bar".into();
        let operations = vec![Operation::Move {
            from: from_path,
            path: to_path,
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidFromPath, err.error_code)
            }
            unexpected => panic!("Expected invalid_source_path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn move_value_inside_array() {
        let initial_json = json!({ "foo": [ "all", "grass", "cows", "eat" ] });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Move {
            from: "/foo/1".into(),
            path: "/foo/3".into(),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
            { "foo": [ "all", "cows", "eat", "grass" ] }
        );
    }

    #[actix_rt::test]
    #[serial]
    async fn move_array_value() {
        let initial_json = json!({
            "foo": ["a", "b", "c"],
            "bar": {
                "baz": ["d"],
                "qux": 1,
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Move {
            from: "/foo/0".into(),
            path: "/bar/baz/1".into(),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": ["b", "c"],
            "bar": {
                "baz": ["d", "a"],
                "qux": 1,
            },
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_copy() {
        let initial_json = json!({
            "foo": { "qux": 1 },
            "bar": {
                "baz": 2,
            },
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Copy {
            from: "/foo/qux".into(),
            path: "/bar/qux".into(),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": { "qux": 1 },
            "bar": {
                "baz": 2,
                "qux": 1,
            },
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_test() {
        let initial_json = json!({
            "foo": 1,
            "bar": 2,
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Test {
            path: "/foo".into(),
            value: Value::Number(1.into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": 1,
            "bar": 2,
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_test_error() {
        let initial_json = json!({
            "foo": 1,
            "bar": 2,
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Test {
            path: "/foo".into(),
            value: Value::Number(99.into()),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::ValueNotEqual, err.error_code)
            }
            unexpected => panic!("Expected failed compare error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn value_is_kept_on_error() {
        let initial_json = json!({
            "foo": 1,
            "bar": 2,
        });

        let (id, mut db_conn) = init_resource(initial_json.clone()).await;

        let operations = vec![
            Operation::Add {
                path: "/baz".into(),
                value: Value::Number(3.into()),
            },
            Operation::Remove {
                path: "/bar".into(),
            },
            Operation::Test {
                path: "/foo".into(),
                value: Value::Number(99.into()),
            },
        ];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::ValueNotEqual, err.error_code)
            }
            unexpected => panic!("Expected failed compare error, got {:?}", unexpected),
        }

        let resource = ModuleResource::get(&mut db_conn, Filter::new().with_id(id))
            .await
            .unwrap()
            .remove(0);

        // resource should not be changed
        assert_eq!(initial_json, resource.data);
    }

    #[actix_rt::test]
    #[serial]
    async fn test_object_value() {
        let initial_json = json!({
        "foo": {
            "baz": 1,
            "bar": 2,
        }});

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Test {
            path: "/foo".into(),
            value: json!({
                "baz": 1,
                "bar": 2,
            }),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": {
                "baz": 1,
                "bar": 2,
            }
        }
        );
    }

    #[actix_rt::test]
    #[serial]
    async fn simple_replace() {
        let initial_json = json!({
            "foo": 1,
            "bar": 2,
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Replace {
            path: "/bar".into(),
            value: Value::Number(99.into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": 1,
            "bar": 99,
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn replace_array_index() {
        let initial_json = json!({
            "foo": ["x", "b", "c"],
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Replace {
            path: "/foo/0".into(),
            value: Value::String("a".into()),
        }];

        let updated = ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        .unwrap();

        assert_eq_json!(updated.data,
        {
            "foo": ["a", "b", "c"],
        });
    }

    #[actix_rt::test]
    #[serial]
    async fn replace_invalid_array_index() {
        let initial_json = json!({
            "foo": ["a", "b", "c"],
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Replace {
            path: "/foo/3".into(),
            value: Value::String("d".into()),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidPath, err.error_code)
            }
            unexpected => panic!("Expected invalid path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn replace_nonexistent_field() {
        let initial_json = json!({
            "foo": 1,
        });

        let (_id, mut db_conn) = init_resource(initial_json).await;

        let operations = vec![Operation::Replace {
            path: "/bar".into(),
            value: Value::String("2".into()),
        }];

        match ModuleResource::patch(
            &mut db_conn,
            Filter::new().with_namespace("test".into()),
            operations,
        )
        .await
        {
            Err(JsonOperationError::JsonPatch(err)) => {
                assert_eq!(JsonPatchErrorCode::InvalidPath, err.error_code)
            }
            unexpected => panic!("Expected invalid path error, got {:?}", unexpected),
        }
    }

    #[actix_rt::test]
    #[serial]
    async fn test_filter() {
        let db_ctx = test_util::database::DatabaseContext::new(false).await;

        let mut db_conn = db_ctx.db.get_conn().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        let room = db_ctx
            .create_test_room(RoomId::from_u128(0), user.id, false)
            .await
            .unwrap();

        let tenant = get_or_create_tenant_by_oidc_id(
            &mut db_conn,
            &OidcTenantId::from("OpenTalkDefaultTenant".to_owned()),
        )
        .await
        .unwrap();

        let new_resource = NewModuleResource {
            tenant_id: tenant.id,
            created_by: user.id,
            room_id: room.id,
            namespace: "test".into(),
            tag: Some("something".into()),
            data: json! {
                {
                    "foo": "a",
                    "bar": "b"
                }
            },
        };

        let module_resource = new_resource.insert(&mut db_conn).await.unwrap();

        let resources = ModuleResource::get(
            &mut db_conn,
            Filter::new().with_namespace("not-test".into()),
        )
        .await
        .unwrap();

        assert!(resources.is_empty());

        let resources = ModuleResource::get(&mut db_conn, Filter::new().with_tag("nothing".into()))
            .await
            .unwrap();

        assert!(resources.is_empty());

        let resources =
            ModuleResource::get(&mut db_conn, Filter::new().with_json(json! {{"baz": "c"}}))
                .await
                .unwrap();

        assert!(resources.is_empty());

        let filter = Filter::new()
            .with_id(module_resource.id)
            .with_created_by(user.id)
            .with_namespace("test".into())
            .with_tag("something".into())
            .with_json(json! {
                {
                    "foo": "a"
                }
            });

        let resources = ModuleResource::get(&mut db_conn, filter).await.unwrap();

        assert!(resources.len() == 1);
        assert_eq!(module_resource, resources[0])
    }
}
