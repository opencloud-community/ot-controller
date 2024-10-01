// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ensure, Snafu};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a recurrence rule
pub const RECURRENCE_RULE_MAX_LEN: usize = 1024;

/// A recurrence rule according to the
/// [`RFC5545`](https://www.rfc-editor.org/rfc/rfc5545) specification.
///
/// Note: currently the rrule patterns are not enforced, the only enforced
/// requirement is a maximum length of [`RECURRENCE_RULE_MAX_LEN`] characters.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct RecurrenceRule(String);

/// An error which can be returned when parsing an a recurrence rule
#[derive(Debug, Snafu)]
pub enum ParseRecurrenceRuleError {
    /// The recurrence rule string is too long
    #[snafu(display(
        "Recurrence rule string is too long. Max length: {max_len}, found length: {found_len}"
    ))]
    RecurrenceRuleTooLong {
        /// The length of the string that was found
        found_len: usize,

        /// The maximum allowed length of the string
        max_len: usize,
    },
}

#[cfg(feature = "utoipa")]
impl<'__s> utoipa::ToSchema<'__s> for RecurrenceRule {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        use serde_json::json;
        (
            "RecurrenceRule",
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::SchemaType::String)
                .max_length(Some(RECURRENCE_RULE_MAX_LEN))
                .description(Some("A recurrence rule according to RFC5545"))
                .example(Some(json!(RecurrenceRule::example_data())))
                .into(),
        )
    }
}

impl FromStr for RecurrenceRule {
    type Err = ParseRecurrenceRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= 1024,
            RecurrenceRuleTooLongSnafu {
                found_len: s.len(),
                max_len: RECURRENCE_RULE_MAX_LEN
            }
        );
        Ok(Self(s.to_string()))
    }
}

impl ExampleData for RecurrenceRule {
    fn example_data() -> Self {
        Self("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO".to_string())
    }
}
