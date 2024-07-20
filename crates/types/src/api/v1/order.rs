// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Ordering query types

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
impl<'__s, T: utoipa::ToSchema<'__s>> utoipa::IntoParams for SortingQuery<T> {
    fn into_params(
        parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
    ) -> Vec<utoipa::openapi::path::Parameter> {
        use utoipa::ToSchema as _;
        vec![
            utoipa::openapi::path::ParameterBuilder::new()
                .name("sort")
                .required(utoipa::openapi::Required::False)
                .parameter_in(parameter_in_provider().unwrap_or_default())
                .description(Some("sort by this field"))
                .schema(Some(utoipa::openapi::Ref::from_schema_name(T::schema().0)))
                .build(),
            utoipa::openapi::path::ParameterBuilder::new()
                .name("order")
                .required(utoipa::openapi::Required::False)
                .parameter_in(parameter_in_provider().unwrap_or_default())
                .description(Some("ordering direction"))
                .schema(Some(utoipa::openapi::Ref::from_schema_name(
                    Ordering::schema().0,
                )))
                .build(),
        ]
    }
}

#[cfg(feature = "utoipa")]
fn print_parameter(p: utoipa::openapi::path::Parameter) {
    let parameter_in = match p.parameter_in {
        utoipa::openapi::path::ParameterIn::Query => "query",
        utoipa::openapi::path::ParameterIn::Path => "path",
        utoipa::openapi::path::ParameterIn::Header => "header",
        utoipa::openapi::path::ParameterIn::Cookie => "cookie",
    };
    let required = match p.required {
        utoipa::openapi::Required::True => "true",
        utoipa::openapi::Required::False => "false",
    };
    let deprecated = p.deprecated.as_ref().map(|d| match d {
        utoipa::openapi::Deprecated::True => "true",
        utoipa::openapi::Deprecated::False => "false",
    });
    let schema = p.schema.as_ref().map(|s| serde_json::to_value(s).unwrap());
    let style = p.style.as_ref().map(|s| match s {
        utoipa::openapi::path::ParameterStyle::Matrix => "matrix",
        utoipa::openapi::path::ParameterStyle::Label => "label",
        utoipa::openapi::path::ParameterStyle::Form => "form",
        utoipa::openapi::path::ParameterStyle::Simple => "simple",
        utoipa::openapi::path::ParameterStyle::SpaceDelimited => "space_delimited",
        utoipa::openapi::path::ParameterStyle::PipeDelimited => "pipe_delimited",
        utoipa::openapi::path::ParameterStyle::DeepObject => "deep_object",
    });

    println!("Parameter {{");
    println!("  name: {:?}", p.name);
    println!("  parameter_in: {parameter_in}");
    println!("  description: {:?}", p.name);
    println!("  required: {}", required);
    println!("  deprecated: {:?}", deprecated);
    println!("  schema: {:?}", schema);
    println!("  style: {:?}", style);
    println!("  explode: {:?}", p.explode);
    println!("  allow_reserved: {:?}", p.allow_reserved);
    println!("  extensions: {:?}", p.extensions);
    println!("}}");
}

#[cfg(feature = "utoipa")]
impl<'__s, T: utoipa::ToSchema<'__s>> SortingQuery<T> {
    /// Will be removed again
    pub fn print_into_params() {
        use utoipa::IntoParams as _;
        for param in SortingQuery::<T>::into_params(|| None) {
            print_parameter(param);
        }
    }
}

/// The sorting order that should be applied
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum Ordering {
    /// Sorting the lowest value first
    Ascending,

    /// Sorting the highest value first
    #[default]
    Descending,
}

/// Properties by which a list of assets can get sorted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum AssetSorting {
    /// Sort by filename
    Filename,

    /// Sort by size
    Size,

    /// Sort by namespace
    Namespace,

    /// Sort by kind
    Kind,

    /// Sort by crated at date
    #[default]
    CreatedAt,
}

#[cfg(test)]
mod test {
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
