// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Data stored inside the `GET /events/{event_id}/instances` query cursor
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetEventInstancesCursorData {
    /// Page number
    pub page: i64,
}

impl ExampleData for GetEventInstancesCursorData {
    fn example_data() -> Self {
        Self { page: 4 }
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use opentalk_types_common::utils::ExampleData as _;
    use serde_json::json;
    use utoipa::{
        openapi::{ObjectBuilder, RefOr, Schema, Type},
        PartialSchema, ToSchema,
    };

    use super::GetEventInstancesCursorData;
    use crate::Cursor;

    impl PartialSchema for GetEventInstancesCursorData {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A cursor pointing to an event instance"))
                .examples([json!(Cursor(Self::example_data()))])
                .into()
        }
    }

    impl ToSchema for GetEventInstancesCursorData {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}
