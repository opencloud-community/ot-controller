// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Pagination types
//!
//! Great blogposts are:
//! - <https://phauer.com/2015/restful-api-design-best-practices/>
//! - <https://phauer.com/2018/web-api-pagination-timestamp-id-continuation-token/>

mod page_pagination_query;

pub use page_pagination_query::PagePaginationQuery;

/// The number of entries per page when using pagination
pub const fn default_pagination_per_page() -> i64 {
    30
}
