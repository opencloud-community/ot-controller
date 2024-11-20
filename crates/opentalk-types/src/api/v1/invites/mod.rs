// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 invites endpoints.

#[allow(unused_imports)]
use crate::imports::*;

mod post_invite_verify_request;

pub use post_invite_verify_request::PostInviteVerifyRequest;
