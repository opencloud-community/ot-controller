// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 sip config endpoints.

#[allow(unused_imports)]
use crate::imports::*;
use crate::{
    core::{CallInId, CallInPassword, RoomId},
    utils::ExampleData,
};

/// Response for the `GET /rooms/{room_id}/sip` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(SipConfigResource::example_data())))]
pub struct SipConfigResource {
    /// The room id
    pub room: RoomId,
    /// The SIP ID
    pub sip_id: CallInId,
    /// The SIP password
    pub password: CallInPassword,
    /// Flag if the room is a lobby
    pub lobby: bool,
}

impl ExampleData for SipConfigResource {
    fn example_data() -> Self {
        Self {
            room: RoomId::example_data(),
            sip_id: CallInId::example_data(),
            password: CallInPassword::example_data(),
            lobby: false,
        }
    }
}

/// Body for the `PUT /rooms/{room_id}/sip` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize, Validate),
    validate(schema(function = "disallow_empty"))
)]
pub struct PutSipConfig {
    /// The SIP password
    #[cfg_attr(feature = "serde", validate(nested))]
    pub password: Option<CallInPassword>,
    /// Flag if the room is a lobby
    pub lobby: Option<bool>,
}

#[cfg(feature = "serde")]
fn disallow_empty(modify_room: &PutSipConfig) -> Result<(), ValidationError> {
    let PutSipConfig { password, lobby } = modify_room;

    if password.is_none() && lobby.is_none() {
        Err(ValidationError::new("empty"))
    } else {
        Ok(())
    }
}
