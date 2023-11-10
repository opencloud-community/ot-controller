// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Trait for augmenting requests with authentication information
pub trait Authorization: std::fmt::Debug {
    /// Augment a request with authorization information
    fn add_authorization_information(&self, request: &mut http::request::Request<Vec<u8>>);
}
