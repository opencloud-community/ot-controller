// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use super::SignalingModulePeerFrontendData;

#[allow(unused_imports)]
use crate::imports::*;

/// A struct containing data of a peer for multiple signaling modules, each
/// associated with the module's namespace.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModulePeerData<K = String>(BTreeMap<K, serde_json::Value>)
where
    K: Ord + From<&'static str>;

impl<K> ModulePeerData<K>
where
    K: Ord + From<&'static str>,
{
    /// Create a new empty [`ModulePeerData`].
    pub fn new() -> Self {
        Self(BTreeMap::default())
    }

    /// Get the peer frontend data for a specific module
    pub fn get<T: SignalingModulePeerFrontendData>(&self) -> Result<Option<T>, serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            self.0
                .get(&K::from(namespace))
                .map(|m| serde_json::from_value(m.clone()))
                .transpose()
        } else {
            Ok(None)
        }
    }

    /// Set the peer frontend data for a specific module
    ///
    /// If an entry with the namespace already exists, it will be overwritten.
    /// If the namespace of `T` is [`None`], the data will not be stored at all.
    pub fn insert<T: SignalingModulePeerFrontendData>(
        &mut self,
        data: &T,
    ) -> Result<(), serde_json::Error> {
        if let Some(namespace) = T::NAMESPACE {
            let _ = self.0.insert(namespace.into(), serde_json::to_value(data)?);
        }
        Ok(())
    }

    /// Query whether the module data contains a value for this key
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q> + Ord,
        Q: Ord,
    {
        self.0.contains_key(key)
    }

    /// Remove an entry by the namespace of [`SignalingModulePeerFrontendData`] type.
    ///
    /// This method does not verify that the data actually can be deserialized
    /// into the requested types, it just uses its namespace and removes an
    /// entry with that namespace if it exists.
    pub fn remove<T: SignalingModulePeerFrontendData>(&mut self) {
        if let Some(namespace) = T::NAMESPACE {
            self.remove_key(&K::from(namespace));
        }
    }

    /// Remove an entry by its key if it exists.
    pub fn remove_key<Q: ?Sized>(&mut self, key: &Q)
    where
        K: std::borrow::Borrow<Q> + Ord,
        Q: Ord,
    {
        let _ = self.0.remove(key);
    }

    /// Take an entry by the namespace of a type.
    ///
    /// If the entry is present but can not be deserialized into the requested type,
    /// the data will remain unchanged, the entry is not removed.
    pub fn take<T: SignalingModulePeerFrontendData>(
        &mut self,
    ) -> Result<Option<T>, serde_json::Error> {
        let entry = self.get::<T>()?;
        self.remove::<T>();
        Ok(entry)
    }
}
