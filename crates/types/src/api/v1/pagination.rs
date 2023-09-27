// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Pagination Query types
//!
//! Great blogposts are: <https://phauer.com/2015/restful-api-design-best-practices/> and <https://phauer.com/2018/web-api-pagination-timestamp-id-continuation-token/>

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
            default = "default_pagination_per_page",
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

/// The number of entries per page when using pagination
pub const fn default_pagination_per_page() -> i64 {
    30
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
        Err(serde::de::Error::custom("page too large"))
    }
}
