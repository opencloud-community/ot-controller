// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Pagination Query types

#[allow(unused_imports)]
use crate::imports::*;

/// Page-based pagination query
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PagePaginationQuery {
    /// The number of entries per page
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "super::default_pagination_per_page",
            deserialize_with = "deserialize_pagination_per_page"
        )
    )]
    pub per_page: i64,

    /// The number of the page
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_pagination_page",
            deserialize_with = "deserialize_pagination_page"
        )
    )]
    pub page: i64,
}

/// Enforce the per_page setting to be <=100 and >0
#[cfg(feature = "serde")]
fn deserialize_pagination_per_page<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let per_page = i64::deserialize(deserializer)?;
    if per_page <= 100 && per_page > 0 {
        Ok(per_page)
    } else if per_page <= 0 {
        Err(serde::de::Error::custom("per_page <= 0"))
    } else {
        Err(serde::de::Error::custom("per_page too large"))
    }
}

#[cfg(feature = "serde")]
const fn default_pagination_page() -> i64 {
    1
}

/// Enforce the page setting to be >0
#[cfg(feature = "serde")]
fn deserialize_pagination_page<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let page = i64::deserialize(deserializer)?;
    if page > 0 {
        Ok(page)
    } else {
        Err(serde::de::Error::custom("page must be greater than 0"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "serde")]
    fn pagination_query() -> Result<(), Box<dyn std::error::Error>> {
        let asset_resource = PagePaginationQuery {
            per_page: 12,
            page: 2,
        };

        let expected_json = "per_page=12&page=2";

        let serialized: String = serde_urlencoded::to_string(&asset_resource)?;
        assert_eq!(expected_json, serialized);

        let deserialized = serde_urlencoded::from_str(expected_json)?;
        assert_eq!(asset_resource, deserialized);

        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn pagination_query_out_of_bounds() {
        use serde::de::Error;

        assert_eq!(
            Err(serde_urlencoded::de::Error::custom(
                "page must be greater than 0"
            )),
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=12&page=-2")
        );
        assert_eq!(
            Err(serde_urlencoded::de::Error::custom("per_page <= 0")),
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=-12&page=2")
        );
        assert_eq!(
            Err(serde_urlencoded::de::Error::custom("per_page too large")),
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=101&page=2")
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn default_page() {
        use crate::api::v1::pagination::default_pagination_per_page;

        let query_default_page = PagePaginationQuery {
            per_page: 12,
            page: default_pagination_page(),
        };

        assert_eq!(
            Ok(query_default_page),
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=12")
        );

        let query_default_page = PagePaginationQuery {
            per_page: default_pagination_per_page(),
            page: 13,
        };

        assert_eq!(
            Ok(query_default_page),
            serde_urlencoded::from_str::<PagePaginationQuery>("page=13")
        );

        let query_default = PagePaginationQuery {
            per_page: default_pagination_per_page(),
            page: default_pagination_page(),
        };
        assert_eq!(
            Ok(query_default),
            serde_urlencoded::from_str::<PagePaginationQuery>("")
        );
    }
}
