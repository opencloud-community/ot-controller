// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use crate::{DestroyContext, Event, InitContext, ModuleContext};

/// Extension to a the signaling websocket
#[async_trait::async_trait(?Send)]
pub trait SignalingModule: Sized + 'static {
    /// Defines the websocket message namespace
    ///
    /// Must be unique between all registered modules.
    const NAMESPACE: &'static str;

    /// The module params, can be any type that is `Clone` + `Send` + `Sync`
    ///
    /// Will get passed to `init` as parameter
    type Params: Clone + Send + Sync;

    /// The websocket incoming message type
    type Incoming: for<'de> Deserialize<'de>;

    /// The websocket outgoing message type
    type Outgoing: Serialize + PartialEq + Debug;

    /// Message type sent over the message exchange to other participant's modules
    type ExchangeMessage: for<'de> Deserialize<'de> + Serialize;

    /// Optional event type, yielded by `ExtEventStream`
    ///
    /// If the module does not register external events it should be set to `()`.
    type ExtEvent;

    /// Data about the owning user of the ws-module which is sent to the frontend on join
    type FrontendData: Serialize;

    /// Data about a peer which is sent to the frontend
    type PeerFrontendData: Serialize;

    /// Constructor of the module
    ///
    /// Provided with the websocket context the modules params and the negotiated protocol
    /// The module can decide to no initiate based on the protocol and passed ctx and params.
    /// E.g. when the user is a bot or guest.
    async fn init(
        ctx: InitContext<'_, Self>,
        params: &Self::Params,
        protocol: &'static str,
    ) -> Result<Option<Self>>;

    /// Events related to this module will be passed into this function together with [`ModuleContext`]
    /// which gives access to the websocket and other related information.
    async fn on_event(
        &mut self,
        ctx: ModuleContext<'_, Self>,
        event: Event<'_, Self>,
    ) -> Result<()>;

    /// Before dropping the module this function will be called
    async fn on_destroy(self, ctx: DestroyContext<'_>);
}
