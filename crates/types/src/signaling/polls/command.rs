// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `polls` namespace

use std::collections::BTreeSet;

use opentalk_types_signaling_polls::{command::Start, ChoiceId, PollId};

#[allow(unused_imports)]
use crate::imports::*;

/// Commands received by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum PollsCommand {
    /// Start a poll
    Start(Start),

    /// Vote in the poll
    Vote(Vote),

    /// Finish the poll
    Finish(Finish),
}

impl From<Start> for PollsCommand {
    fn from(value: Start) -> Self {
        Self::Start(value)
    }
}

/// Command to vote in the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vote {
    /// The id of the poll
    pub poll_id: PollId,

    /// The choices
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub choices: Choices,
}

/// The choices of a vote
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum Choices {
    /// A single choice. Takes precedence over `Multiple` during deserialization
    Single {
        /// The choice id
        choice_id: ChoiceId,
    },
    /// A multiple choice, `choice_ids` might be empty to abstain
    Multiple {
        /// The set of choice ids
        #[cfg_attr(feature = "serde", serde(default))]
        choice_ids: BTreeSet<ChoiceId>,
    },
}

impl Choices {
    /// Returns the choices as a BTreeSet
    pub fn to_hash_set(self) -> BTreeSet<ChoiceId> {
        match self {
            Self::Single { choice_id } => BTreeSet::from_iter(vec![choice_id]),
            Self::Multiple { choice_ids } => choice_ids,
        }
    }
}

/// Command to finish the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Finish {
    /// The id of the poll
    pub id: PollId,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn start() {
        let json = json!({
            "action": "start",
            "topic": "abc",
            "live": true,
            "choices": ["a", "b", "c"],
            "duration": 30
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Start(Start {
            topic,
            live,
            multiple_choice,
            choices,
            duration,
        }) = message
        {
            assert_eq!(topic, "abc");
            assert!(live);
            assert!(!multiple_choice);
            assert_eq!(choices, vec!["a", "b", "c"]);
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!()
        }
    }

    #[test]
    fn single_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn multiple_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(322), ChoiceId::from(323)]),
                }
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn conflicting_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn abstain() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000"
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::new(),
                }
            );
        } else {
            panic!()
        }
    }
}
