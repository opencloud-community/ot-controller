// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use types::{
    core::ParticipantId,
    signaling::{control::state::ControlState, Role},
};

use crate::SignalingModule;

/// Event passed to [`SignalingModule::on_event`]
pub enum Event<'evt, M>
where
    M: SignalingModule,
{
    /// The participant joined the room
    Joined {
        /// Data set by the control module. Some modules require attributes specified by the
        /// control module which are provided here on join
        control_data: &'evt ControlState,

        /// The module can set this option to Some(M::FrontendData) to populate
        /// the `join_success` message with additional information to the frontend module counterpart
        frontend_data: &'evt mut Option<M::FrontendData>,

        /// List of participants already inside the room.
        ///
        /// The module can populate participant specific frontend-data, which is sent inside
        /// the participant inside the `join_success` message
        participants: &'evt mut HashMap<ParticipantId, Option<M::PeerFrontendData>>,
    },

    /// The participant is in the process of leaving the room, this event will be called before
    /// `on_destroy` is called and before the exchange control message `Left` has been sent.
    ///
    /// Note: Calls to `ModuleContext::ws_send` when receiving this event will almost certainly fail
    Leaving,

    /// A user can request attention by 'raising' his hand, this event gets broadcast to every
    /// module.
    RaiseHand,

    /// User lowered his hand and no longer requests attention.
    LowerHand,

    /// Participant with the associated id has joined the room
    ParticipantJoined(ParticipantId, &'evt mut Option<M::PeerFrontendData>),

    /// Participant with the associated id has left the room
    ParticipantLeft(ParticipantId),

    /// Participant data has changed, an options to `M::PeerFrontendData`
    ParticipantUpdated(ParticipantId, &'evt mut Option<M::PeerFrontendData>),

    /// Role of the participant changed
    RoleUpdated(Role),

    /// Received websocket message
    WsMessage(M::Incoming),

    /// Exchange subscriber received a message for this module
    Exchange(M::ExchangeMessage),

    /// External event provided by eventstream which was added using [`crate::InitContext::add_event_stream`].
    ///
    /// Modules that didn't register external events will
    /// never receive this variant and can ignore it.
    Ext(M::ExtEvent),
}
