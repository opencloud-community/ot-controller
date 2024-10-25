// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    event::{Error, Started},
    Results,
};

/// Events sent out by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
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

impl From<Started> for PollsEvent {
    fn from(value: Started) -> Self {
        Self::Started(value)
    }
}

impl From<Error> for PollsEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{Choice, ChoiceId, Item, PollId};

    #[test]
    fn started() {
        let started = PollsEvent::Started(Started {
            id: PollId::nil(),
            topic: "polling".into(),
            live: true,
            multiple_choice: false,
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
                "multiple_choice": false,
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
