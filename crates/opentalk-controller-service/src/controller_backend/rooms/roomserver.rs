// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides roomserver-related implementation

use chrono::Utc;
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_database::{DatabaseError, DbConnection};
use opentalk_db_storage::{
    events::{Event, shared_folders::EventSharedFolder},
    invites::Invite,
    sip_configs::SipConfig,
    streaming_targets::RoomStreamingTargetRecord,
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    room_parameters::{EventContext, RoomParameters},
};
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::{
        RoomResource,
        by_room_id::{
            PostRoomsRoomserverStartInvitedRequestBody, PostRoomsRoomserverStartRequestBody,
            RoomserverStartResponseBody,
        },
    },
    users::PublicUserProfile,
};
use opentalk_types_common::{
    call_in::CallInInfo,
    rooms::RoomId,
    roomserver::Token as RoomserverToken,
    shared_folders::{SharedFolder, SharedFolderAccess},
    streaming::StreamingLink,
    users::UserInfo,
};

use crate::{ControllerBackend, controller_backend::rooms::start_room_error::StartRoomError};

impl ControllerBackend {
    pub(crate) async fn roomserver_start_room(
        &self,
        user: RequestUser,
        room_id: RoomId,
        request: PostRoomsRoomserverStartRequestBody,
    ) -> Result<RoomserverStartResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();

        let Some(roomserver_address) = settings
            .roomserver
            .as_ref()
            .map(|config| config.url.to_string())
        else {
            return Err(StartRoomError::RoomserverSignalingDisabled.into());
        };

        let room = self.get_room(&room_id).await?;

        let role = if room.created_by.id == user.id {
            Role::Moderator
        } else {
            Role::User
        };

        let client_parameters = ClientParameters {
            device_secret: request.device_secret,
            kind: ClientKind::Registered {
                profile: PublicUserProfile {
                    id: user.id,
                    email: user.email,
                    user_info: UserInfo {
                        title: user.title,
                        firstname: user.firstname,
                        lastname: user.lastname,
                        display_name: request.display_name.unwrap_or(user.display_name),
                        avatar_url: user
                            .avatar_url
                            .unwrap_or(settings.avatar.libravatar_url.clone()),
                    },
                },
            },
            role,
        };

        let token = self
            .request_roomserver_token(room, client_parameters)
            .await?;

        Ok(RoomserverStartResponseBody {
            token,
            roomserver_address,
        })
    }

    pub(crate) async fn roomserver_start_room_invited(
        &self,
        room_id: RoomId,
        request: PostRoomsRoomserverStartInvitedRequestBody,
    ) -> Result<RoomserverStartResponseBody, CaptureApiError> {
        let Some(roomserver_address) = self
            .settings_provider
            .get()
            .roomserver
            .as_ref()
            .map(|config| config.url.to_string())
        else {
            return Err(StartRoomError::RoomserverSignalingDisabled.into());
        };

        self.authenticate_guest(&room_id, &request.invite_code, &request.password)
            .await?;

        let room = self.get_room(&room_id).await?;

        let client_parameters = ClientParameters {
            device_secret: request.device_secret,
            kind: ClientKind::Guest {
                display_name: request.display_name,
            },
            role: Role::User,
        };

        let token = self
            .request_roomserver_token(room, client_parameters)
            .await?;

        Ok(RoomserverStartResponseBody {
            token,
            roomserver_address,
        })
    }

    async fn request_roomserver_token(
        &self,
        room: RoomResource,
        client_parameters: ClientParameters,
    ) -> Result<RoomserverToken, ApiError> {
        let room_id = room.id;

        let Some(client) = &self.roomserver_client else {
            return Err(ApiError::internal()
                .with_message("roomserver is not configured on this controller"));
        };

        let maybe_token = client
            .request_token(room_id, client_parameters.clone(), None)
            .await
            .map_err(|e| {
                log::error!("Failed to request token from roomserver for room `{room_id}`: {e}");

                ApiError::internal().with_message("failed to request token from roomserver")
            })?;

        let token = match maybe_token {
            Some(token) => token,
            None => {
                // The room is unknown to the roomserver,- resubmit the token request but include the room parameter
                let room_parameters = self.build_room_parameters(room).await?;

                let final_response = client
                    .request_token(room_id, client_parameters, Some(room_parameters))
                    .await
                    .map_err(|e| {
                        log::error!("failed to request token from roomserver: {e}");

                        ApiError::internal().with_message("failed to request token from roomserver")
                    })?;

                let Some(token) = final_response else {
                    log::error!(
                        "Roomserver responded with 'unknown room' despite the request containing room information"
                    );
                    return Err(ApiError::internal()
                        .with_message("unable to request token from roomserver"));
                };

                token
            }
        };

        Ok(token)
    }

    async fn build_room_parameters(
        &self,
        room: RoomResource,
    ) -> Result<RoomParameters, CaptureApiError> {
        let mut conn = self.db.get_conn().await?;
        let settings = self.settings_provider.get();

        let call_in = Self::get_call_in_info(&mut conn, &settings, room.id).await?;

        let event = Self::get_event_context(&mut conn, room.id).await?;

        let streaming_links = Self::build_streaming_links(&mut conn, room.id).await?;

        let invite_code = Invite::get_valid_for_room(&mut conn, room.id, Utc::now())
            .await?
            .map(|invite| invite.id);

        let tariff = self.get_room_tariff(&room.id).await?;

        let parameters = RoomParameters {
            created_by: room.created_by,
            password: room.password,
            waiting_room: room.waiting_room,
            call_in,
            event,
            invite_code,
            tariff,
            streaming_links,
            e2e_encryption: false,
        };

        Ok(parameters)
    }

    async fn get_event_context(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Option<EventContext>, CaptureApiError> {
        let Some(event) = Event::get_for_room(conn, room_id).await? else {
            return Ok(None);
        };

        let shared_folder = match EventSharedFolder::get_for_event(conn, event.id).await? {
            Some(event_shared_folder) => Some(SharedFolder {
                read: SharedFolderAccess {
                    url: event_shared_folder.read_url,
                    password: event_shared_folder.read_password,
                },
                read_write: Some(SharedFolderAccess {
                    url: event_shared_folder.write_url,
                    password: event_shared_folder.write_password,
                }),
            }),
            None => None,
        };

        let context = EventContext {
            id: event.id,
            title: event.title,
            description: event.description,
            is_adhoc: event.is_adhoc,
            shared_folder,
        };

        Ok(Some(context))
    }

    async fn get_call_in_info(
        conn: &mut DbConnection,
        settings: &Settings,
        room_id: RoomId,
    ) -> Result<Option<CallInInfo>, CaptureApiError> {
        let Some(tel) = settings.call_in.as_ref().map(|call_in| call_in.tel.clone()) else {
            return Ok(None);
        };

        match SipConfig::get_by_room(conn, room_id).await {
            Ok(sip_config) => Ok(Some(CallInInfo {
                tel,
                id: sip_config.sip_id,
                password: sip_config.password,
            })),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(err) => Err(CaptureApiError::from(err)),
        }
    }

    async fn build_streaming_links(
        conn: &mut DbConnection,
        room_id: RoomId,
    ) -> Result<Vec<StreamingLink>, CaptureApiError> {
        let streaming_targets = RoomStreamingTargetRecord::get_all_for_room(conn, room_id).await?;
        let mut streaming_links = Vec::new();

        for target in streaming_targets {
            let url = match target.public_url.parse() {
                Ok(url) => url,
                Err(err) => {
                    log::warn!(
                        "Unparsable streaming url in streaming records for room {room_id}: {err}"
                    );
                    continue;
                }
            };

            streaming_links.push(StreamingLink {
                name: target.name,
                url,
            });
        }

        Ok(streaming_links)
    }
}
