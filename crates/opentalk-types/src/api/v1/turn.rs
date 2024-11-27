// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 TURN endpoints.

use opentalk_types_api_v1::turn::IceServer;

#[allow(unused_imports)]
use crate::imports::*;

/// Response to the *GET /turn* endpoint request
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetTurnServersResponse(pub Vec<IceServer>);
