// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

/// This type is required to properly serialize the `Event`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaybeUserName {
    name: Option<DisplayName>,
}

impl From<Option<DisplayName>> for MaybeUserName {
    fn from(name: Option<DisplayName>) -> Self {
        Self { name }
    }
}
