// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Some helper utilities for interacting with the data storage.

use opentalk_db_storage::{
    events::{Event, EventAndEncryption},
    tariffs::Tariff,
};
use opentalk_types_common::{
    call_in::CallInInfo,
    events::{EventInfo, MeetingDetails},
    features,
    rooms::RoomId,
    streaming::get_public_urls_from_room_streaming_targets,
};

use crate::{Inventory, Result};

/// Build the user-facing event info for a room.
pub async fn build_event_info(
    inventory: &mut dyn Inventory,
    call_in_tel: Option<String>,
    room_id: RoomId,
    e2e_encryption: bool,
    event: &Event,
    tariff: &Tariff,
) -> Result<EventInfo> {
    let event_info = if event.show_meeting_details {
        let invite = inventory.get_first_invite_for_room(room_id).await?;

        let call_in = if let Some(call_in_tel) = call_in_tel {
            if e2e_encryption || tariff.is_feature_disabled(&features::CALL_IN_MODULE_FEATURE_ID) {
                None
            } else {
                inventory
                    .get_room_sip_config(room_id)
                    .await?
                    .map(|sip_config| CallInInfo {
                        tel: call_in_tel,
                        id: sip_config.sip_id,
                        password: sip_config.password,
                    })
            }
        } else {
            None
        };

        let streaming_links = if !e2e_encryption {
            let streaming_targets = inventory.get_room_streaming_targets(room_id).await?;
            get_public_urls_from_room_streaming_targets(streaming_targets).await
        } else {
            vec![]
        };

        EventInfo::from(EventAndEncryption(event, e2e_encryption)).with_meeting_details(
            MeetingDetails {
                invite_code_id: invite.map(|invite| invite.id),
                call_in,
                streaming_links,
            },
        )
    } else {
        EventInfo::from(EventAndEncryption(event, e2e_encryption))
    };

    Ok(event_info)
}
