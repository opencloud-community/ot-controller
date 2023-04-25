// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Choice, PollId};
use serde::Serialize;
use std::time::Duration;
use types::signaling::polls::{event::Error, Results};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum PollsEvent {
    Started(Started),
    LiveUpdate(Results),
    Done(Results),
    Error(Error),
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Started {
    pub id: PollId,
    pub topic: String,
    pub live: bool,
    pub choices: Vec<Choice>,
    #[serde(with = "super::duration_secs")]
    pub duration: Duration,
}

#[cfg(test)]
mod test {
    use super::*;
    use test_util::assert_eq_json;
    use types::signaling::polls::{ChoiceId, Item};

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

        assert_eq_json!(
          started,
          {
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
          }
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

        assert_eq_json!(
          live_update,
          {
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
          }
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

        assert_eq_json!(
          done,
          {
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
          }
        );
    }
}
