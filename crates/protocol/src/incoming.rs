// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use types::signaling::protocol::command::ParticipantSelection;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum ProtocolCommand {
    SelectWriter(ParticipantSelection),
    DeselectWriter(ParticipantSelection),
    /// Generates a pdf of the current protocol contents.
    GeneratePdf,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use types::core::ParticipantId;

    #[test]
    fn select_writer() {
        let json_str = r#"
        {
            "action": "select_writer",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        }
        "#;

        if let ProtocolCommand::SelectWriter(ParticipantSelection { participant_ids }) =
            serde_json::from_str(json_str).unwrap()
        {
            assert_eq!(participant_ids[0], ParticipantId::from_u128(0));
            assert_eq!(participant_ids[1], ParticipantId::from_u128(1));
        } else {
            panic!("expected SelectWriter variant");
        }
    }

    #[test]
    fn deselect_writer() {
        let json_str = r#"
        {
            "action": "deselect_writer",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        }
        "#;

        if let ProtocolCommand::DeselectWriter(ParticipantSelection { participant_ids }) =
            serde_json::from_str(json_str).unwrap()
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
