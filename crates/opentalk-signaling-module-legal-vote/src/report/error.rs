// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportGenerationError;
use opentalk_types_common::users::UserId;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("Database error"), context(false))]
    Database {
        source: opentalk_database::DatabaseError,
    },

    #[snafu(display("Failed to generate the report"))]
    ReportGeneration { source: ReportGenerationError },

    #[snafu(display("The legal vote protocol is missing the start entry"))]
    MissingStartEntry,

    #[snafu(display("The legal vote protocol is missing the stop entry"))]
    MissingStopEntry,

    #[snafu(display("Display name for user id {user_id} not found"))]
    UserDisplayNameNotFound { user_id: UserId },
}
