// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Debug, io::Write};

use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{IsNull, ToSql},
    sql_types,
};
use opentalk_database::{DatabaseError, DbConnection};
use opentalk_types::common::{
    event::{EventInfo, MeetingDetails},
    streaming::get_public_urls_from_streaming_targets,
};
use opentalk_types_common::{call_in::CallInInfo, rooms::RoomId, users::UserId};
use serde::{Deserialize, Serialize};

use crate::{
    events::{Event, EventAndEncryption},
    invites::Invite,
    sip_configs::SipConfig,
    streaming_targets::get_room_streaming_targets,
};

/// Trait for models that have user-ids attached to them like created_by/updated_by fields
///
/// Used to make batch requests of users after fetching some resources
///
/// Should only be implemented on references of the actual models
pub trait HasUsers {
    fn populate(self, dst: &mut Vec<UserId>);
}

impl<T, I> HasUsers for I
where
    T: HasUsers,
    I: IntoIterator<Item = T>,
{
    fn populate(self, dst: &mut Vec<UserId>) {
        for t in self {
            t.populate(dst);
        }
    }
}

/// JSONB Wrapper for any type implementing the serde `Serialize` or `Deserialize` trait
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, FromSqlRow, AsExpression, PartialEq, Eq,
)]
#[diesel(sql_type = sql_types::Jsonb)]
pub struct Jsonb<T>(pub T);

impl<T: for<'de> Deserialize<'de>> FromSql<sql_types::Jsonb, Pg> for Jsonb<T> {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes = value.as_bytes();
        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }
        serde_json::from_slice(&bytes[1..])
            .map(Self)
            .map_err(|_| "Invalid Json".into())
    }
}

impl<T: Serialize + Debug> ToSql<sql_types::Jsonb, Pg> for Jsonb<T> {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, &self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

pub async fn build_event_info(
    conn: &mut DbConnection,
    call_in_tel: Option<String>,
    room_id: RoomId,
    e2e_encrytion: bool,
    event: &Event,
) -> Result<EventInfo, DatabaseError> {
    let event_info = if event.show_meeting_details {
        let invite = Invite::get_first_for_room(conn, room_id).await?;

        let call_in = if let Some(call_in_tel) = call_in_tel {
            match SipConfig::get_by_room(conn, room_id).await {
                Ok(sip_config) => Some(CallInInfo {
                    tel: call_in_tel,
                    id: sip_config.sip_id,
                    password: sip_config.password,
                }),
                Err(DatabaseError::NotFound) => None,
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        let streaming_targets = get_room_streaming_targets(conn, room_id).await?;
        let streaming_links = get_public_urls_from_streaming_targets(streaming_targets).await;

        EventInfo::from(EventAndEncryption(event, e2e_encrytion)).with_meeting_details(
            MeetingDetails {
                invite_code_id: invite.map(|invite| invite.id),
                call_in,
                streaming_links,
            },
        )
    } else {
        EventInfo::from(EventAndEncryption(event, e2e_encrytion))
    };

    Ok(event_info)
}
