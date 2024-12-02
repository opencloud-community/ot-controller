// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling users.

mod display_name;
mod group_id;
mod group_name;
mod language;
mod theme;
mod user_id;
mod user_title;

pub use display_name::DisplayName;
pub use group_id::GroupId;
pub use group_name::GroupName;
pub use language::Language;
pub use theme::Theme;
pub use user_id::UserId;
pub use user_title::{ParseUserTitleError, UserTitle, MAX_USER_TITLE_LENGTH};
