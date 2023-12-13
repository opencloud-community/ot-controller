// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Deref, DerefMut};

#[allow(unused_imports)]
use crate::imports::*;

/// Opaque token which represents T as a base64 string (where T is encoded using bincode)
///
/// Used for cursor based pagination
#[derive(Deref, DerefMut, AsRef, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cursor<T>(pub T);

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;

    use std::marker::PhantomData;

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    impl<T> Cursor<T>
    where
        T: Serialize,
    {
        /// Encode T using bincode and return it as base64 string
        pub fn to_base64(&self) -> String {
            URL_SAFE_NO_PAD.encode(bincode::serialize(&self.0).unwrap())
        }
    }

    impl<T> Serialize for Cursor<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_base64())
        }
    }

    impl<'de, T> Deserialize<'de> for Cursor<T>
    where
        T: DeserializeOwned,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(CursorVisitor::<T>(PhantomData))
        }
    }

    struct CursorVisitor<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for CursorVisitor<T>
    where
        T: DeserializeOwned,
    {
        type Value = Cursor<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 + bincode encoded cursor data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let bytes = URL_SAFE_NO_PAD
                .decode(v)
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))?;
            let data = bincode::deserialize(&bytes)
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Bytes(&bytes), &self))?;

            Ok(Cursor(data))
        }
    }
}
