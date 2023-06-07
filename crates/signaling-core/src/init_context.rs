// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{marker::PhantomData, sync::Arc};

use database::Db;
use db_storage::{rooms::Room, users::User};
use futures::{stream::SelectAll, Stream};
use kustos::Authz;
use types::{
    core::{BreakoutRoomId, ParticipantId},
    signaling::Role,
};

use crate::{
    any_stream, AnyStream, ObjectStorage, Participant, RedisConnection, SignalingModule,
    SignalingRoomId,
};

pub struct ExchangeBinding {
    pub routing_key: String,
}

/// Context passed to the `init` function
pub struct InitContext<'ctx, M>
where
    M: SignalingModule,
{
    pub id: ParticipantId,
    pub room: &'ctx Room,
    pub breakout_room: Option<BreakoutRoomId>,
    pub participant: &'ctx Participant<User>,
    pub role: Role,
    pub db: &'ctx Arc<Db>,
    pub storage: &'ctx Arc<ObjectStorage>,
    pub authz: &'ctx Arc<Authz>,
    pub exchange_bindings: &'ctx mut Vec<ExchangeBinding>,
    pub events: &'ctx mut SelectAll<AnyStream>,
    pub redis_conn: &'ctx mut RedisConnection,
    pub m: PhantomData<fn() -> M>,
}

impl<M> InitContext<'_, M>
where
    M: SignalingModule,
{
    /// ID of the participant the module instance belongs to
    pub fn participant_id(&self) -> ParticipantId {
        self.id
    }

    /// Returns a reference to the database representation of the the room
    ///
    /// Note that the room will always be the same regardless if inside a
    /// breakout room or not.
    pub fn room(&self) -> &Room {
        self.room
    }

    /// ID of the room currently inside, this MUST be used when a module does not care about
    /// whether it is inside a breakout room or not.
    pub fn room_id(&self) -> SignalingRoomId {
        SignalingRoomId::new(self.room.id, self.breakout_room)
    }

    /// Returns the ID of the breakout room, if inside one
    pub fn breakout_room(&self) -> Option<BreakoutRoomId> {
        self.breakout_room
    }

    /// Returns the user associated with the participant
    pub fn participant(&self) -> &Participant<User> {
        self.participant
    }

    /// Returns the role of participant inside the room
    pub fn role(&self) -> Role {
        self.role
    }

    /// Returns a reference to the controllers database interface
    pub fn db(&self) -> &Arc<Db> {
        self.db
    }

    /// Returns a reference to the controllers S3 storage interface
    pub fn storage(&self) -> &Arc<ObjectStorage> {
        self.storage
    }

    pub fn authz(&self) -> &Arc<Authz> {
        self.authz
    }

    /// Access to a redis connection
    pub fn redis_conn(&mut self) -> &mut RedisConnection {
        self.redis_conn
    }

    /// Add a routing-key for the exchange-subscriber to bind to
    pub fn add_exchange_binding(&mut self, routing_key: String) {
        self.exchange_bindings.push(ExchangeBinding { routing_key });
    }

    /// Add a custom event stream which return `M::ExtEvent`
    pub fn add_event_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = M::ExtEvent> + 'static,
    {
        self.events.push(any_stream(M::NAMESPACE, stream));
    }
}
