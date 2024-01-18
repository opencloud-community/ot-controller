// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 streaming service endpoints.

use crate::{common::streaming::StreamingService, core::StreamingServiceId};

#[allow(unused_imports)]
use crate::imports::*;

/// The parameter set for /users/me/streaming_services/{streaming_service_id}* endpoints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamingServiceIdentifier {
    /// The streaming service id
    pub streaming_service_id: StreamingServiceId,
}

/// The body of a *GET /users/me/streaming_services* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetStreamingServicesResponse(pub Vec<StreamingService>);

/// The body of a *GET /users/me/streaming_services/{streaming_service_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GetStreamingServiceResponse(pub StreamingService);
