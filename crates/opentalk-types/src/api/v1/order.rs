// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Ordering query types

use opentalk_types_common::order::Ordering;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Order query type
///
/// The struct describes the query parameter that can be provided to sort the returned collection.
/// The generic parameter T describes the options by which the collection can get sorted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortingQuery<T> {
    /// The optional sorting query parameter
    #[cfg_attr(feature = "serde", serde(default))]
    pub sort: T,

    /// The sorting order that should be applied to the collection
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: Ordering,
}

// `#[derive(utoipa::IntoParams)]` attempts to reference `#/components/schemas/T`,
// which is typically not the correct target for the `sort` parameter. This
// custom implementation resolves the sort target type correctly.
#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use utoipa::{openapi::path::ParameterBuilder, PartialSchema as _, ToSchema};

    use super::SortingQuery;
    use crate::api::v1::order::Ordering;

    impl<T: ToSchema> utoipa::IntoParams for SortingQuery<T> {
        fn into_params(
            parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
        ) -> Vec<utoipa::openapi::path::Parameter> {
            vec![
                ParameterBuilder::new()
                    .name("sort")
                    .required(utoipa::openapi::Required::False)
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                    .description(Some("sort by this field"))
                    .schema(Some(T::schema()))
                    .build(),
                ParameterBuilder::new()
                    .name("order")
                    .required(utoipa::openapi::Required::False)
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                    .description(Some("ordering direction"))
                    .schema(Some(Ordering::schema()))
                    .build(),
            ]
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use opentalk_types_common::assets::AssetSorting;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn asset_sort_query() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let paging = SortingQuery {
            sort: AssetSorting::CreatedAt,
            order: Ordering::Descending,
        };

        let paging_urlencoded = "sort=created_at&order=descending";

        let serialize_output: String = serde_urlencoded::to_string(paging)?;
        assert_eq!(paging_urlencoded, serialize_output);

        let deserialized = serde_urlencoded::from_str(paging_urlencoded)?;
        assert_eq!(paging, deserialized);

        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn invalid_asset_sort_query() {
        use serde::de::Error;

        assert_eq!(
            serde_urlencoded::from_str::<SortingQuery<AssetSorting>>("sort=wrong_field"),
            Err(serde_urlencoded::de::Error::custom(
                "unknown variant `wrong_field`, expected one of `filename`, `size`, `namespace`, `kind`, `created_at`"
            )),
        );
        assert_eq!(
            serde_urlencoded::from_str::<SortingQuery<AssetSorting>>("order=wrong_order"),
            Err(serde_urlencoded::de::Error::custom(
                "unknown variant `wrong_order`, expected `ascending` or `descending`"
            )),
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn asset_query_default_values() {
        let default_ordering = SortingQuery::<AssetSorting>::default();

        assert_eq!(
            Ok(default_ordering),
            serde_urlencoded::from_str::<SortingQuery<AssetSorting>>("")
        );
    }
}
