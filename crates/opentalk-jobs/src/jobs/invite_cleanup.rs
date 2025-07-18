// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use kustos::Authz;
use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::room::associated_resource_ids_for_invite;
use opentalk_database::Db;
use opentalk_db_storage::invites::Invite;
use opentalk_log::{debug, info};
use opentalk_signaling_core::ExchangeHandle;
use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InviteCleanupParameters {
    expired_before: Option<Timestamp>,
}

impl JobParameters for InviteCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job for cleaning up expired invites
#[derive(Debug)]
pub struct InviteCleanup;

#[async_trait]
impl Job for InviteCleanup {
    type Parameters = InviteCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        db: Arc<Db>,
        _exchange_handle: ExchangeHandle,
        _settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Executing invite cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");
        info!(log: logger, "");

        let mut conn = db.get_conn().await?;

        let authz = Authz::new(db.clone()).await?;

        let expired_before = parameters.expired_before.unwrap_or_else(|| {
            info!(log: logger, "Parameter field expired_before not set. Using current timestamp.");
            Timestamp::now()
        });

        info!(log: logger, "Clearing permissions for invites that are inactive or expired before {expired_before:?}.");

        let inactive_invites =
            Invite::get_inactive_or_expired_before(&mut conn, expired_before.into()).await?;
        let mut count = 0;

        for (invite_code, room_id) in inactive_invites {
            let associated_resources = Vec::from_iter(associated_resource_ids_for_invite(room_id));
            let deleted = authz
                .remove_all_invite_permission_for_resources(invite_code, associated_resources)
                .await?;
            if deleted != 0 {
                count += 1;
            }
        }

        info!(log: logger, "Number of cleared invites: {count}");

        Ok(())
    }
}
