// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Deref, DerefMut};

/// Arbitrary data that can be wrapped inside a [`Cursor`].
pub trait CursorData: std::fmt::Debug {
    /// The name of the cursor type which will be referenced in the OpenAPI specification.
    const SCHEMA_CURSOR_TYPE_NAME: &'static str;
}

/// Opaque token which represents T as a base64 string (where T is encoded using bincode)
///
/// Used for cursor based pagination
#[derive(Deref, DerefMut, AsRef, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cursor<T: CursorData>(pub T);

#[cfg(feature = "utoipa")]
impl<'__s, T: CursorData> utoipa::ToSchema<'__s> for Cursor<T> {
    fn schema() -> (
        &'__s str,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    ) {
        (
            T::SCHEMA_CURSOR_TYPE_NAME,
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::SchemaType::String)
                .description(Some(
                    "An encoded cursor pointing to a position in a series of elements",
                ))
                .into(),
        )
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use std::marker::PhantomData;

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl<T: CursorData + Serialize> Cursor<T> {
        /// Encode T using bincode and return it as base64 string
        pub fn to_base64(&self) -> String {
            URL_SAFE_NO_PAD.encode(bincode::serialize(&self.0).unwrap())
        }
    }

    impl<T: CursorData + Serialize> Serialize for Cursor<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_base64())
        }
    }

    impl<'de, T: CursorData + DeserializeOwned> Deserialize<'de> for Cursor<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(CursorVisitor::<T>(PhantomData))
        }
    }

    struct CursorVisitor<T: CursorData>(PhantomData<T>);

    impl<'de, T: CursorData + DeserializeOwned> serde::de::Visitor<'de> for CursorVisitor<T> {
        type Value = Cursor<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 + bincode encoded cursor data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let bytes = URL_SAFE_NO_PAD.decode(v).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
            })?;
            let data = bincode::deserialize(&bytes).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Bytes(&bytes), &self)
            })?;

            Ok(Cursor(data))
        }
    }
}
