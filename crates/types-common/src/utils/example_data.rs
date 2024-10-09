// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// A trait for providing example data of an item.
pub trait ExampleData {
    /// Get an example instance of the current datatype.
    fn example_data() -> Self;
}
