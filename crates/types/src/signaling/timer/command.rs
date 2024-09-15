// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `timer` namespace

use opentalk_types_signaling_timer::{
    command::{Start, Stop},
    TimerId,
};

#[allow(unused_imports)]
use crate::imports::*;

/// Incoming websocket messages
#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize),
    serde(rename_all = "snake_case", tag = "action")
)]
pub enum Message {
    /// Start a new timer
    Start(Start),
    /// Stop a running timer
    Stop(Stop),
    /// Update the ready status
    UpdateReadyStatus(UpdateReadyStatus),
}

/// Update the ready status
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct UpdateReadyStatus {
    /// The timer id
    pub timer_id: TimerId,
    /// The new status
    pub status: bool,
}

#[cfg(test)]
mod tests {
    use opentalk_types_signaling_timer::command::Kind;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn countdown_start() {
        let json = json!({
            "action": "start",
            "kind": "countdown",
            "duration": 5,
            "style": "coffee_break",
            "enable_ready_check": false
        });

        match serde_json::from_value(json).unwrap() {
            Message::Start(Start {
                kind,
                style,
                title,
                enable_ready_check,
            }) => {
                assert_eq!(kind, Kind::Countdown { duration: 5 });
                assert_eq!(style, Some("coffee_break".into()));
                assert_eq!(title, None);
                assert!(!enable_ready_check);
            }
            unexpected => panic!("Expected start message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn stopwatch_start() {
        let json = json!({
            "action": "start",
            "kind": "stopwatch",
            "title": "Testing the timer!",
            "enable_ready_check": false
        });

        match serde_json::from_value(json).unwrap() {
            Message::Start(Start {
                kind,
                style,
                title,
                enable_ready_check,
            }) => {
                assert_eq!(kind, Kind::Stopwatch);
                assert_eq!(style, None);
                assert_eq!(title, Some("Testing the timer!".into()));
                assert!(!enable_ready_check);
            }
            unexpected => panic!("Expected start message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn stop() {
        let json = json!({
            "action": "stop",
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "reason": "test"
        });

        match serde_json::from_value(json).unwrap() {
            Message::Stop(Stop { timer_id, reason }) => {
                assert_eq!(reason, Some("test".into()));
                assert_eq!(timer_id, TimerId::nil())
            }
            unexpected => panic!("Expected stop message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn update_ready_status() {
        let json = json!({
            "action": "update_ready_status",
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "status": true
        });

        match serde_json::from_value(json).unwrap() {
            Message::UpdateReadyStatus(UpdateReadyStatus { timer_id, status }) => {
                assert!(status);
                assert_eq!(timer_id, TimerId::nil())
            }
            unexpected => panic!("Expected ready message, got: {unexpected:?}"),
        }
    }
}
