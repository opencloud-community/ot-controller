// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_legal_vote::{
    event::{Canceled, FinalResults, PdfAsset, ReportedIssue, StopKind, Stopped},
    parameters::Parameters,
    token::Token,
    vote::{LegalVoteId, VoteOption},
};
use serde::{Deserialize, Serialize};

/// Rabbitmq event to inform participants
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// A new vote has started
    Start(Parameters),
    /// A participant has successfully voted, the message gets dispatched to the underlying user id
    Voted(VoteSuccess),
    /// A vote has been stopped
    Stop(Stopped),
    /// A vote has been canceled
    Cancel(Canceled),
    /// The results for a vote have changed
    Update(VoteUpdate),
    /// A participant reported an issue
    Issue(ReportedIssue),
    /// A fatal internal server error has occurred
    FatalServerError,

    PdfAsset(PdfAsset),
}

/// A participant has successfully voted
///
/// This gets send to all participants that are participating with the same underlying user_id
#[derive(Debug, Serialize, Deserialize)]
pub struct VoteSuccess {
    /// The vote id
    pub legal_vote_id: LegalVoteId,
    /// The participant that issued the vote cast
    pub issuer: ParticipantId,
    /// The chosen vote option
    pub vote_option: VoteOption,
    /// The token that is used to cast the vote
    pub consumed_token: Token,
}

/// The specified vote has been stopped
#[derive(Debug, Serialize, Deserialize)]
pub struct Stop {
    /// The id of the stopped vote
    pub legal_vote_id: LegalVoteId,
    /// The kind of stop
    #[serde(flatten)]
    pub kind: StopKind,
    /// The final vote results
    pub results: FinalResults,
}

/// The results for a vote have changed
#[derive(Debug, Serialize, Deserialize)]
pub struct VoteUpdate {
    /// The id of the affected vote
    pub legal_vote_id: LegalVoteId,
}
