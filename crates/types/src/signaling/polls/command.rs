// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `polls` namespace

use std::time::Duration;

#[allow(unused_imports)]
use crate::imports::*;

use super::{ChoiceId, PollId};

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

/// Command to start a poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Start {
    /// The description of the poll topic
    pub topic: String,

    /// Is the poll live
    pub live: bool,

    /// The choices of the poll
    pub choices: Vec<String>,

    /// The duration of the poll
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    pub duration: Duration,
}

/// Command to vote in the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vote {
    /// The id of the poll
    pub poll_id: PollId,

    /// The id of the choice or `None` to abstain
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub choice_id: Option<ChoiceId>,
}

/// Command to finish the poll
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Finish {
    /// The id of the poll
    pub id: PollId,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
            choices,
            duration,
        }) = message
        {
            assert_eq!(topic, "abc");
            assert!(live);
            assert_eq!(choices, vec!["a", "b", "c"]);
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!()
        }
    }

    #[test]
    fn vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choice_id }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(choice_id, Some(ChoiceId::from(321)));
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

        if let PollsCommand::Vote(Vote { poll_id, choice_id }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(choice_id, None);
        } else {
            panic!()
        }
    }
}
