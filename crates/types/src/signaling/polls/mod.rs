// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling messages for the `polls` namespace

mod choice;
mod choice_id;
mod item;
mod poll_id;
mod results;

pub mod command;
pub mod event;
pub mod state;

pub use choice::Choice;
pub use choice_id::ChoiceId;
pub use item::Item;
pub use poll_id::PollId;
pub use results::Results;

/// The namespace string for the signaling module
pub const NAMESPACE: &str = "polls";
