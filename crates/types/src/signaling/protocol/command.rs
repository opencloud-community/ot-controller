// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use crate::core::ParticipantId;

#[allow(unused_imports)]
use crate::imports::*;

/// Commands for the `protocol` namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "action", rename_all = "snake_case")
)]
pub enum ProtocolCommand {
    /// Select a participant as writer
    SelectWriter(ParticipantSelection),

    /// Deselect a participant as writer
    DeselectWriter(ParticipantSelection),

    /// Generates a pdf of the current protocol contents
    GeneratePdf,
}

/// Give a list of participants write access to the protocol
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct ParticipantSelection {
    /// The targeted participants
    pub participant_ids: Vec<ParticipantId>,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn select_writer() {
        let json_str = json!(
        {
            "action": "select_writer",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        });

        if let ProtocolCommand::SelectWriter(ParticipantSelection { participant_ids }) =
            serde_json::from_value(json_str).unwrap()
        {
            assert_eq!(participant_ids[0], ParticipantId::from_u128(0));
            assert_eq!(participant_ids[1], ParticipantId::from_u128(1));
        } else {
            panic!("expected SelectWriter variant");
        }
    }

    #[test]
    fn deselect_writer() {
        let json_str = json!(
        {
            "action": "deselect_writer",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        });

        if let ProtocolCommand::DeselectWriter(ParticipantSelection { participant_ids }) =
            serde_json::from_value(json_str).unwrap()
        {
            assert_eq!(participant_ids[0], ParticipantId::from_u128(0));
            assert_eq!(participant_ids[1], ParticipantId::from_u128(1));
        } else {
            panic!("expected SelectWriter variant");
        }
    }

    #[test]
    fn generate_pdf() {
        let json = serde_json::json!({
            "action": "generate_pdf"
        });

        if let ProtocolCommand::GeneratePdf = serde_json::from_value(json).unwrap() {
        } else {
            panic!("expected GeneratePdf variant");
        }
    }
}
