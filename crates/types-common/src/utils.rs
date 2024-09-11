// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Helpful utilities used in this crate, but also useful outside of it.

/// A trait for providing example data of an item.
pub trait ExampleData {
    /// Get an example instance of the current datatype.
    fn example_data() -> Self;
}
