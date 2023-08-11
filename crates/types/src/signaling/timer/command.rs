// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `timer` namespace

#[allow(unused_imports)]
use crate::imports::*;

use super::TimerId;

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

/// The different timer variations
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize),
    serde(rename_all = "snake_case", tag = "kind")
)]
pub enum Kind {
    /// The timer continues to run until a moderator stops it.
    Stopwatch,
    /// The timer continues to run until its duration expires or if a moderator stops it beforehand.
    Countdown {
        /// The duration of the countdown
        duration: u64,
    },
}

/// Start a new timer
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct Start {
    /// The timer kind
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Kind,
    /// An optional string tag to flag this timer with a custom style
    pub style: Option<String>,
    /// An optional title for the timer
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    #[cfg_attr(feature = "serde", serde(default))]
    pub enable_ready_check: bool,
}

/// Stop a running timer
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct Stop {
    /// The timer id
    pub timer_id: TimerId,
    /// An optional reason for the stop
    pub reason: Option<String>,
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
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
