// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::RoomPassword;

#[allow(unused_imports)]
use crate::imports::*;

/// API request parameters to patch a room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize,))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PatchRoomsRequestBody {
    /// The password for the room
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "::serde_with::rust::double_option",
        )
    )]
    pub password: Option<Option<RoomPassword>>,

    /// If waiting room is enabled
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub waiting_room: Option<bool>,

    /// If e2e encryption is enabled
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub e2e_encryption: Option<bool>,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = PatchRoomsRequestBody {
            password: None,
            waiting_room: None,
            e2e_encryption: None,
        };
        let json = json!({});

        let serialized = serde_json::to_value(&body)?;
        assert_eq!(serialized, json);

        let deserialized: PatchRoomsRequestBody = serde_json::from_value(json)?;
        assert_eq!(deserialized, body);

        Ok(())
    }

    #[test]
    fn full() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = PatchRoomsRequestBody {
            password: Some(Some("hello".parse()?)),
            waiting_room: Some(false),
            e2e_encryption: Some(true),
        };
        let json = json!({
            "password": "hello",
            "waiting_room": false,
            "e2e_encryption": true,
        });

        let serialized = serde_json::to_value(&body)?;
        assert_eq!(serialized, json);

        let deserialized: PatchRoomsRequestBody = serde_json::from_value(json)?;
        assert_eq!(deserialized, body);

        Ok(())
    }

    #[test]
    fn reset_password() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = PatchRoomsRequestBody {
            password: Some(None),
            waiting_room: None,
            e2e_encryption: None,
        };
        let json = json!({
            "password": null
        });

        let serialized = serde_json::to_value(&body)?;
        assert_eq!(serialized, json);

        let deserialized: PatchRoomsRequestBody = serde_json::from_value(json)?;
        assert_eq!(deserialized, body);

        Ok(())
    }
}
