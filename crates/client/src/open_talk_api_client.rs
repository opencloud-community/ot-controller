// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::error::Error;

use shared::{ApiError, Authorization, WithAuthorization};
use types::{
    api::v1::{
        auth::{GetLoginRequest, OidcProvider},
        invites::{CodeVerified, VerifyBody},
        rooms::GetRoomEventRequest,
    },
    common::event::EventInfo,
    core::{InviteCodeId, RoomId},
};

#[async_trait::async_trait]
pub trait OpenTalkApiClient<E>
where
    E: Error + Send + Sync + 'static,
{
    async fn get_login(&self) -> Result<OidcProvider, ApiError<E>>;
    async fn post_invite_verify(
        &self,
        invite_code: InviteCodeId,
    ) -> Result<CodeVerified, ApiError<E>>;
    async fn get_room_event<A: Authorization + Send>(
        &self,
        authorization: A,
        room: RoomId,
    ) -> Result<EventInfo, ApiError<E>>;
}

#[async_trait::async_trait]
impl<C: shared::Client + Sync> OpenTalkApiClient<C::Error> for C {
    async fn get_login(&self) -> Result<OidcProvider, ApiError<C::Error>> {
        self.rest(GetLoginRequest)
            .await
            .map(|response| response.oidc)
    }

    async fn post_invite_verify(
        &self,
        invite_code: InviteCodeId,
    ) -> Result<CodeVerified, ApiError<C::Error>> {
        self.rest(VerifyBody { invite_code }).await
    }

    async fn get_room_event<A: Authorization + Send>(
        &self,
        authorization: A,
        room_id: RoomId,
    ) -> Result<EventInfo, ApiError<C::Error>> {
        self.rest(GetRoomEventRequest(room_id).with_authorization(authorization))
            .await
            .map(|result| result.0)
    }
}
