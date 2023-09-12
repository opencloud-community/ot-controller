// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `polls` namespace

use std::time::Duration;

#[allow(unused_imports)]
use crate::imports::*;

use super::{Choice, PollId, Results};

/// Events sent out by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum PollsEvent {
    /// The poll has started
    Started(Started),

    /// Live update of the poll results
    LiveUpdate(Results),

    /// The poll is completed
    Done(Results),

    /// An error happened when executing a `polls` command
    Error(Error),
}

/// Event signaling to the participant that the poll has started
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Started {
    /// The id of the poll
    pub id: PollId,

    /// The description of the poll topic
    pub topic: String,

    /// Is the poll live
    pub live: bool,

    /// Choices of the poll
    pub choices: Vec<Choice>,

    /// Duration of the poll
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    pub duration: Duration,
}

impl From<Started> for PollsEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

/// Errors from the `polls` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "error")
)]
pub enum Error {
    /// Attempted to perform a command which requires more permissions
    InsufficientPermissions,

    /// Attempted to start a poll with invalid choice count
    InvalidChoiceCount,

    /// Attempted to perform a command with an invalid poll id
    InvalidPollId,

    /// Attempted to perform a command with an invalid choice id
    InvalidChoiceId,

    /// Attempted to perform a command with an invalid choice description
    InvalidChoiceDescription,

    /// Attempted to perform a command with an invalid duration
    InvalidDuration,

    /// Attempted to perform a command with an invalid topic length
    InvalidTopicLength,

    /// Attempted to start a new poll while an existing one is still running
    StillRunning,
}

impl From<Error> for PollsEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod test {
    use crate::signaling::polls::{ChoiceId, Item};

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn started() {
        let started = PollsEvent::Started(Started {
            id: PollId::nil(),
            topic: "polling".into(),
            live: true,
            choices: vec![
                Choice {
                    id: ChoiceId::from(0),
                    content: "yes".into(),
                },
                Choice {
                    id: ChoiceId::from(1),
                    content: "no".into(),
                },
            ],
            duration: Duration::from_millis(10000),
        });

        assert_eq!(
            serde_json::to_value(started).unwrap(),
            json!({
                "message": "started",
                "id": "00000000-0000-0000-0000-000000000000",
                "topic": "polling",
                "live": true,
                "choices": [
                    {
                        "id": 0,
                        "content": "yes"
                    },
                    {
                        "id": 1,
                        "content": "no"
                    }
                ],
                "duration": 10
            })
        );
    }

    #[test]
    fn live_update() {
        let live_update = PollsEvent::LiveUpdate(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(
            serde_json::to_value(live_update).unwrap(),
            json!({
                "message": "live_update",
                "id": "00000000-0000-0000-0000-000000000000",
                "results": [
                    {
                        "id": 0,
                        "count": 32
                    },
                    {
                        "id": 1,
                        "count": 64
                    }
                ]
            })
        );
    }

    #[test]
    fn done() {
        let done = PollsEvent::Done(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(
            serde_json::to_value(done).unwrap(),
            json!({
                "message": "done",
                "id": "00000000-0000-0000-0000-000000000000",
                "results": [
                    {
                        "id": 0,
                        "count": 32
                    },
                    {
                        "id": 1,
                        "count": 64
                    }
                ]
            })
        );
    }
}
