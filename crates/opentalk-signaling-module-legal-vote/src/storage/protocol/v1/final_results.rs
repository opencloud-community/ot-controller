// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling final results for the `legal-vote` namespace.

use std::collections::BTreeSet;

use opentalk_types_common::users::UserId;
use opentalk_types_signaling_legal_vote::{invalid::Invalid, tally::Tally};

/// Final vote results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "results")]
pub enum FinalResults {
    /// Valid vote results.
    Valid(Tally),

    /// Invalid vote results.
    Invalid(Invalid),
}

impl FinalResults {
    /// Creates a empty [`BTreeSet`].
    ///
    /// Always returns a empty set. This is used in [`crate::storage::protocol::v1::VoteEvent`].
    pub fn get_referenced_user_ids(&self) -> BTreeSet<UserId> {
        BTreeSet::new()
    }
}

#[cfg(test)]
mod serde_tests {

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization_valid_final_results() {
        let produced = serde_json::to_value(FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        }))
        .unwrap();

        let expected = json!({
            "results": "valid",
            "yes": 1,
            "no": 0,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_valid_final_results() {
        let produced: FinalResults = serde_json::from_value(json!({
            "results": "valid",
            "yes": 1,
            "no": 0,
        }))
        .unwrap();

        let expected = FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialization_invalid_final_results() {
        let produced =
            serde_json::to_value(FinalResults::Invalid(Invalid::AbstainDisabled)).unwrap();

        let expected = json!({
            "results": "invalid",
            "reason": "abstain_disabled",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization_invalid_final_results() {
        let produced: FinalResults = serde_json::from_value(json!({
            "results": "invalid",
            "reason": "abstain_disabled",
        }))
        .unwrap();

        let expected = FinalResults::Invalid(Invalid::AbstainDisabled);

        assert_eq!(produced, expected);
    }
}
