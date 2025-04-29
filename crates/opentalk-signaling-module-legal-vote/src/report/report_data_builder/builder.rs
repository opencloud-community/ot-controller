// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use chrono_tz::Tz;
use opentalk_report_generation::{ReportDateTime, ToReportDateTime as _};
use opentalk_types_common::users::{DisplayName, UserId};
use snafu::OptionExt as _;

use super::{StopInfo, VoteData};
use crate::{
    report::{
        data::{
            Event, ReportData, ResolvedCancel, ResolvedReportedIssue, ResolvedVote, StopReason,
            TimedEvent,
        },
        error::UserDisplayNameNotFoundSnafu,
        Error,
    },
    storage::v1::{
        Cancel, FinalResults, MaybeUserInfo, ProtocolEntry, ReportedIssue, Start, StopKind, Vote,
        VoteEvent,
    },
};

pub struct Builder {
    user_names: BTreeMap<UserId, DisplayName>,
    data: VoteData,
}

impl Builder {
    pub(crate) fn new(user_names: BTreeMap<UserId, DisplayName>) -> Self {
        Self {
            user_names,
            data: VoteData::default(),
        }
    }

    pub(crate) fn build_report_data(
        mut self,
        protocol: Vec<ProtocolEntry>,
        timezone: &Tz,
    ) -> Result<ReportData, Error> {
        for ProtocolEntry { timestamp, event } in protocol {
            let time = timestamp.to_report_date_time(timezone);
            self.handle_event(event, time)?;
        }

        self.data.finalize(&self.user_names, timezone)
    }

    fn handle_event(
        &mut self,
        event: VoteEvent,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        match event {
            VoteEvent::Start(start) => self.handle_start(start),
            VoteEvent::Vote(vote) => self.handle_vote(vote, time)?,
            VoteEvent::Stop(stop_kind) => self.handle_stop(stop_kind, time)?,
            VoteEvent::FinalResults(final_results) => self.handle_final_results(final_results),
            VoteEvent::Issue(reported_issue) => self.handle_issue(reported_issue, time)?,
            VoteEvent::UserLeft(user_info) => self.handle_user_left(user_info, time)?,
            VoteEvent::UserJoined(user_info) => self.handle_user_joined(user_info, time)?,
            VoteEvent::Cancel(cancel) => self.handle_cancel(cancel, time)?,
        }

        Ok(())
    }

    fn handle_start(&mut self, start: Start) {
        self.data.start = Some(start)
    }

    fn handle_vote(&mut self, vote: Vote, time: Option<ReportDateTime>) -> Result<(), Error> {
        let name = match vote.user_info {
            Some(info) => Some(
                self.user_names
                    .get(&info.issuer)
                    .context(UserDisplayNameNotFoundSnafu {
                        user_id: info.issuer,
                    })?
                    .clone(),
            ),
            None => None,
        };

        self.data.votes.push(ResolvedVote {
            name,
            token: vote.token.to_string(),
            option: vote.option,
            time,
        });

        Ok(())
    }

    fn handle_stop(
        &mut self,
        stop_kind: StopKind,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let stop_kind = match stop_kind {
            StopKind::ByUser(user_id) => StopReason::ByUser {
                user: self.get_user_name(user_id)?,
            },
            StopKind::Auto => StopReason::Auto,
            StopKind::Expired => StopReason::Expired,
        };

        self.data.stop_info = Some(StopInfo {
            time,
            reason: stop_kind,
        });

        Ok(())
    }

    fn handle_final_results(&mut self, final_results: FinalResults) {
        self.data.final_results = Some(final_results)
    }

    fn handle_issue(
        &mut self,
        ReportedIssue { user_info, issue }: ReportedIssue,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::Issue(ResolvedReportedIssue { name, issue }),
        });

        Ok(())
    }

    fn handle_user_left(
        &mut self,
        user_info: MaybeUserInfo,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info.inner {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::UserLeft(name.into()),
        });

        Ok(())
    }

    fn handle_user_joined(
        &mut self,
        user_info: MaybeUserInfo,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info.inner {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::UserJoined(name.into()),
        });

        Ok(())
    }

    fn handle_cancel(&mut self, cancel: Cancel, time: Option<ReportDateTime>) -> Result<(), Error> {
        self.data.stop_info = Some(StopInfo {
            time,
            reason: StopReason::Canceled(ResolvedCancel {
                user: self.get_user_name(cancel.issuer)?,
                reason: cancel.reason,
            }),
        });

        Ok(())
    }

    fn get_user_name(&self, user_id: UserId) -> Result<DisplayName, Error> {
        self.user_names
            .get(&user_id)
            .context(UserDisplayNameNotFoundSnafu { user_id })
            .cloned()
    }
}
