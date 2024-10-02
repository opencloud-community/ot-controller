// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `meeting_report` namespace

/// Incoming websocket messages
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case", tag = "action")
)]
pub enum Message {
    /// Generate a report on current and past meeting attendees
    GenerateAttendanceReport {
        /// Include
        include_email_addresses: bool,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn create_attendees_report() {
        let json = json!({
            "action": "generate_attendance_report",
            "include_email_addresses": false,
        });

        assert_eq!(
            serde_json::from_value::<Message>(json).unwrap(),
            Message::GenerateAttendanceReport {
                include_email_addresses: false
            }
        );
    }
}
