// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Trait for augmenting requests with authentication information
pub trait Authorization: std::fmt::Debug {
    /// Add authorization headers
    fn apply_authorization_headers(&self, headers: &mut http::HeaderMap);
}
