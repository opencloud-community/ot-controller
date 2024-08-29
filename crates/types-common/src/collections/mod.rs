// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Collection data types.

pub mod one_or_many_btree_set;

#[cfg(feature = "serde")]
pub use one_or_many_btree_set::one_or_many_btree_set_option;
pub use one_or_many_btree_set::OneOrManyBTreeSet;
