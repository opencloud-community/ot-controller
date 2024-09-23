// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/auth`.

pub mod login;

mod oidc_provider;

pub use oidc_provider::OidcProvider;
