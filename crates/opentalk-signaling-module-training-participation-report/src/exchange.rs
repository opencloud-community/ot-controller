// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types sent inside the signaling module between different runner instances

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling_training_participation_report::event::{
    PdfAsset, PresenceLoggingEndedReason, PresenceLoggingStartedReason,
};
use serde::{Deserialize, Serialize};

/// An event sent between the runner instances
#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    /// Presence logging has started.
    PresenceLoggingStarted {
        /// The timestamp of the first checkpoint
        first_checkpoint: Timestamp,

        /// The reason for starting the presence logging
        reason: PresenceLoggingStartedReason,
    },

    /// Presence logging has ended.
    PresenceLoggingEnded {
        /// The reason why the presence logging has ended.
        reason: PresenceLoggingEndedReason,
    },

    /// Presence logging has been enabled by the room owner.
    PresenceLoggingEnabled,

    /// Presence logging has been disabled by the room owner.
    PresenceLoggingDisabled,

    /// Presence confirmation requested.
    PresenceConfirmationRequested,

    /// Hand the responsibility for the presence logging over to a different room owner participant runner
    RoomOwnerHandOver {
        /// The next checkpoint timestamp, used to determine the timer that needs to be set in the runner
        next_checkpoint: Timestamp,
    },

    /// A PDF asset has been created, all participants of the room owner are informed.
    PdfAsset(PdfAsset),
}
