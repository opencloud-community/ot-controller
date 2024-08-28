// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub(crate) mod resumption;
pub(crate) mod storage;
pub(crate) mod ticket;

mod ws;
mod ws_modules;

use itertools::Itertools;
pub use ws::SignalingModules;
pub(crate) use ws::{SignalingProtocols, __path_ws_service, ws_service};
pub use ws_modules::{breakout, echo, moderation, recording, recording_service};

/// Trim leading, trailing, and extra whitespaces between a given display name.
fn trim_display_name(display_name: String) -> String {
    display_name.split_whitespace().join(" ")
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::trim_display_name;

    #[test]
    fn trim_display_name_leading_spaces() {
        assert_eq!("First Last", trim_display_name("  First Last".to_string()));
    }

    #[test]
    fn trim_display_name_trailing_spaces() {
        assert_eq!("First Last", trim_display_name("First Last  ".to_string()));
    }

    #[test]
    fn trim_display_name_spaces_between() {
        assert_eq!("First Last", trim_display_name("First  Last".to_string()));
    }
}
