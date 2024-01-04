// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "kustos")]
use kustos_shared::subject::PolicyInvite;

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

#[allow(unused_imports)]
use crate::imports::*;

/// An invite code
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(
    feature = "diesel",
    derive(DieselNewtype, AsExpression, FromSqlRow),
    diesel(sql_type = diesel::sql_types::Uuid),
)]
#[cfg_attr(feature = "kustos", derive(KustosPrefix), kustos_prefix("/invites/"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InviteCodeId(Uuid);

impl InviteCodeId {
    /// Create a ZERO invite code id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a invite code id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random invite code id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[cfg(feature = "kustos")]
impl From<InviteCodeId> for PolicyInvite {
    fn from(id: InviteCodeId) -> Self {
        Self::from(Uuid::from(id))
    }
}

#[cfg(feature = "frontend")]
impl client_shared::Authorization for InviteCodeId {
    fn add_authorization_information(&self, request: &mut http::request::Request<Vec<u8>>) {
        let _ = request.headers_mut().insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_str(&format!("InviteCode {}", self))
                .expect("valid header value"),
        );
    }
}

#[cfg(feature = "actix")]
mod actix_impls {
    use std::str::FromStr;

    use super::*;

    use actix_http::header::{HeaderValue, InvalidHeaderValue, TryIntoHeaderValue};
    use actix_web_httpauth::headers::authorization::{ParseError, Scheme};
    use bytes::{BufMut, BytesMut};

    const IDENTIFIER_LENGTH: usize = 10;
    const SPACE_LENGTH: usize = 1;
    const UUID_LENGTH: usize = 36;
    const BUFFER_LENGTH: usize = IDENTIFIER_LENGTH + SPACE_LENGTH + UUID_LENGTH;

    impl TryIntoHeaderValue for InviteCodeId {
        type Error = InvalidHeaderValue;

        fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
            let mut buffer = BytesMut::with_capacity(BUFFER_LENGTH);
            buffer.put(&b"InviteCode "[..]);
            let uuid_string = self.to_string();
            buffer.extend_from_slice(uuid_string.as_bytes());

            HeaderValue::from_maybe_shared(buffer.freeze())
        }
    }

    impl Scheme for InviteCodeId {
        fn parse(header: &HeaderValue) -> Result<Self, ParseError> {
            if header.len() < BUFFER_LENGTH {
                return Err(ParseError::Invalid);
            }

            let mut parts = header.to_str()?.splitn(2, ' ');

            match parts.next() {
                Some("InviteCode") => {}
                _ => return Err(ParseError::MissingScheme),
            }

            let invite_code_str = parts.next().ok_or(ParseError::Invalid)?;
            InviteCodeId::from_str(invite_code_str).map_err(|_| ParseError::Invalid)
        }
    }
}

#[cfg(all(test, feature = "actix"))]
mod actix_tests {
    use actix_http::header::{HeaderValue, TryIntoHeaderValue};
    use actix_web_httpauth::headers::authorization::Scheme;

    use super::*;

    #[test]
    fn test_parse_header() {
        let uuid = uuid::uuid!("4bf587d9-1c92-427f-9bf1-522455f93382");
        let code = InviteCodeId::from(uuid);
        let value = HeaderValue::from_str(&format!("InviteCode {}", code)).unwrap();
        let scheme = InviteCodeId::parse(&value);

        assert!(scheme.is_ok());
        let scheme = scheme.unwrap();
        assert_eq!(scheme, code);
    }

    #[test]
    fn test_empty_header() {
        let value = HeaderValue::from_static("");
        let scheme = InviteCodeId::parse(&value);

        assert!(scheme.is_err());
    }

    #[test]
    fn test_wrong_scheme() {
        let value = HeaderValue::from_static("Bearer foo");
        let scheme = InviteCodeId::parse(&value);

        assert!(scheme.is_err());
    }

    #[test]
    fn test_missing_token() {
        let value = HeaderValue::from_static("Bearer ");
        let scheme = InviteCodeId::parse(&value);

        assert!(scheme.is_err());
    }

    #[test]
    fn test_into_header_value() {
        let uuid = uuid::uuid!("4bf587d9-1c92-427f-9bf1-522455f93382");
        let code = InviteCodeId::from(uuid);

        let result = code.try_into_value();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            HeaderValue::from_str(&format!("InviteCode {}", code)).unwrap()
        );
    }
}
