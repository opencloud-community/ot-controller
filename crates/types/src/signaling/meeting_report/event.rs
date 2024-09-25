// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `meeting_report` namespace

use opentalk_types_common::assets::AssetId;

#[allow(unused_imports)]
use crate::imports::*;

/// Events sent out by the `meeting_report` module
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case", tag = "message")
)]
pub enum MeetingReportEvent {
    /// A PDF asset has been created
    PdfAsset(PdfAsset),

    /// An error happened when executing a `meeting_report` command
    Error(Error),
}

/// Handle to a PDF asset
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfAsset {
    /// The file name of the PDF asset
    pub filename: String,

    /// The asset id for the PDF asset
    pub asset_id: AssetId,
}

impl From<PdfAsset> for MeetingReportEvent {
    fn from(value: PdfAsset) -> Self {
        Self::PdfAsset(value)
    }
}

/// Error from the `meeting_report` module namespace
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "error", rename_all = "snake_case")
)]
pub enum Error {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The requesting user has exceeded their storage
    StorageExceeded,
    /// Internal error while generating the report
    Generate,
    /// Internal error while saving the report
    Storage,
}

impl From<Error> for MeetingReportEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::assets::AssetId;
    use serde_json::json;

    use crate::signaling::meeting_report::event::{Error, MeetingReportEvent, PdfAsset};

    #[test]
    fn serialize_meeting_report_event_pdf_asset() {
        let pdf_event = MeetingReportEvent::PdfAsset(PdfAsset {
            filename: "pdf-file.pdf".to_owned(),
            asset_id: AssetId::from_u128(0x735fcdaa_56dd_4ddb_9eb0_7d083a4a9d9b),
        });
        let value = serde_json::to_value(pdf_event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "filename": "pdf-file.pdf",
                "asset_id": "735fcdaa-56dd-4ddb-9eb0-7d083a4a9d9b",
                "message": "pdf_asset",
            })
        );
    }

    #[test]
    fn serialize_meeting_report_event_error() {
        let pdf_event = MeetingReportEvent::Error(Error::Generate);
        let value = serde_json::to_value(pdf_event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "error": "generate",
                "message": "error",
            })
        );
    }
}
