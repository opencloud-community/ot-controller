// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `breakout` namespace

use opentalk_types_signaling_breakout::command::Start;

#[allow(unused_imports)]
use crate::imports::*;

/// Commands for breakout sessions
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum BreakoutCommand {
    /// Command for starting a breakout session
    Start(Start),
    /// Command for stopping a breakout session
    Stop,
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_breakout::command::RoomParameter;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn breakout_start() {
        let json = json!({
            "action": "start",
            "rooms": [
                { "name": "Room 1", "assignments": [], },
                { "name": "Room 2", "assignments": ["00000000-0000-0000-0000-000000000000"], },
            ],
            "duration": 123454321,
        });

        let msg: BreakoutCommand = serde_json::from_value(json).unwrap();

        match msg {
            BreakoutCommand::Start(Start { rooms, duration }) => {
                assert_eq!(
                    rooms,
                    vec![
                        RoomParameter {
                            name: "Room 1".into(),
                            assignments: vec![],
                        },
                        RoomParameter {
                            name: "Room 2".into(),
                            assignments: vec![ParticipantId::nil()],
                        }
                    ]
                );
                assert_eq!(duration, Some(Duration::from_secs(123454321)));
            }
            BreakoutCommand::Stop => panic!(),
        }
    }
}
