// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_signaling_core::{SignalingModuleError, SignalingRoomId, VolatileStorage};
use opentalk_types_signaling::ParticipantId;
use opentalk_types_signaling_legal_vote::{
    event::{Results, VotingRecord},
    state::LegalVoteState,
    token::Token,
    vote::{LegalVoteId, VoteKind, VoteOption, VoteState, VoteSummary},
};
use snafu::{OptionExt, ResultExt, Snafu, ensure};

use crate::{LegalVoteStorageProvider, storage::protocol as db_protocol};

pub struct RawProtocol<'a>(&'a [db_protocol::v1::ProtocolEntry]);

/// Error when converting from `&[ProtocolEntry]` to [`VoteSummary`].
#[derive(Debug, Snafu)]
pub enum TryIntoVoteSummaryError {
    /// Error indicating a missing `Stop` entry before `FinalResults` in the protocol.
    #[snafu(display("Missing `Stop` entry before `FinalResults` in legal vote protocol"))]
    MissingStopEntry,

    /// Error indicating a missing `Start` entry in the legal vote protocol.
    #[snafu(display("Missing `Start` in legal vote protocol"))]
    MissingStartEntry,

    /// Error indicating a missing `VoteState` entry in the legal vote protocol.
    #[snafu(display("Missing `VoteState` in legal vote protocol"))]
    MissingState,

    /// Error indicating a missing `VoteState` entry in the legal vote protocol.
    #[snafu(display("Error while converting `Protocol` to `VotingRecord`"))]
    TryIntoVotingRecord { source: TryIntoVotingRecordError },
}

impl<'a, T> From<&'a T> for RawProtocol<'a>
where
    T: AsRef<[db_protocol::v1::ProtocolEntry]> + ?Sized,
{
    fn from(value: &'a T) -> Self {
        Self(value.as_ref())
    }
}

impl TryFrom<RawProtocol<'_>> for VoteSummary {
    type Error = TryIntoVoteSummaryError;

    fn try_from(protocol: RawProtocol) -> Result<Self, Self::Error> {
        let mut parameters = None;
        let mut state = None;
        let mut end_time = None;
        let mut stop_kind = None;

        for entry in protocol.0 {
            match entry.event.clone() {
                db_protocol::v1::VoteEvent::Start(start) => {
                    parameters = Some(start.parameters);
                    state = Some(VoteState::Started);
                }

                db_protocol::v1::VoteEvent::Stop(kind) => {
                    stop_kind = Some(kind);
                    end_time = entry.timestamp;
                }

                db_protocol::v1::VoteEvent::Cancel(cancel) => {
                    let cancel = cancel.into();
                    state = Some(VoteState::Canceled(cancel));
                    end_time = entry.timestamp;
                }

                db_protocol::v1::VoteEvent::FinalResults(results) => match results {
                    db_protocol::v1::FinalResults::Valid(tally) => {
                        let voting_record =
                            (&protocol).try_into().context(TryIntoVotingRecordSnafu)?;

                        let stop_kind = stop_kind.clone().context(MissingStopEntrySnafu)?.into();

                        state = Some(VoteState::Finished {
                            stop_kind,
                            results: Results {
                                tally,
                                voting_record,
                            },
                        });
                    }

                    db_protocol::v1::FinalResults::Invalid(reason) => {
                        state = Some(VoteState::Invalid(reason));
                    }
                },
                _ => {}
            }
        }

        let parameters = parameters.context(MissingStartEntrySnafu)?;

        let state = state.context(MissingStateSnafu)?;

        Ok(Self {
            parameters,
            state,
            end_time,
        })
    }
}

/// Errors that can occur when parsing a `VotingRecord`.
#[derive(Snafu, Debug)]
pub enum TryIntoVotingRecordError {
    /// The voting protocol contains conflicting or inconsistent entries.
    #[snafu(display("Protocol contains inconsistent entries."))]
    InconsistentEntries,

    /// The voting protocol is missing the required `Start` entry.
    #[snafu(display("Missing `Start` entry in the legal vote protocol."))]
    MissingStart,
}

impl TryFrom<&RawProtocol<'_>> for VotingRecord {
    type Error = TryIntoVotingRecordError;

    fn try_from(protocol: &RawProtocol) -> Result<Self, Self::Error> {
        let kind = protocol
            .0
            .iter()
            .find_map(|entry| match &entry.event {
                db_protocol::v1::VoteEvent::Start(start) => Some(start.parameters.inner.kind),
                _ => None,
            })
            .context(MissingStartSnafu)?;

        let vote_iter = protocol.0.iter().filter_map(|entry| match &entry.event {
            db_protocol::v1::VoteEvent::Vote(vote) => Some(vote),
            _ => None,
        });

        match kind {
            VoteKind::RollCall | VoteKind::LiveRollCall => {
                let voters = vote_iter
                    .map(|vote| {
                        let user_info = vote.user_info.context(InconsistentEntriesSnafu)?;
                        Ok((user_info.participant_id, vote.option))
                    })
                    .collect::<Result<HashMap<ParticipantId, VoteOption>, Self::Error>>()?;
                Ok(Self::UserVotes(voters))
            }

            VoteKind::Pseudonymous => {
                let tokens = vote_iter
                    .map(|vote| {
                        ensure!(vote.user_info.is_none(), InconsistentEntriesSnafu);
                        Ok((vote.token, vote.option))
                    })
                    .collect::<Result<HashMap<Token, VoteOption>, Self::Error>>()?;
                Ok(Self::TokenVotes(tokens))
            }
        }
    }
}

pub async fn load_from_protocol(
    mut volatile: VolatileStorage,
    room_id: SignalingRoomId,
    vote_id: LegalVoteId,
) -> Result<VoteSummary, SignalingModuleError> {
    let storage = volatile.storage();
    let storage_protocol = storage.protocol_get(room_id, vote_id).await?;
    let protocol = RawProtocol::from(&storage_protocol);

    let vote_summary = protocol
        .try_into()
        .map_err(|err| SignalingModuleError::CustomError {
            message: "Failed to summarize protocol".to_string(),
            source: Some(Box::new(err)),
        })?;

    Ok(vote_summary)
}

pub async fn load_from_history(
    mut volatile: VolatileStorage,
    room_id: SignalingRoomId,
    current_vote: Option<LegalVoteId>,
) -> Result<LegalVoteState, SignalingModuleError> {
    let storage = volatile.storage();
    let vote_futures = storage
        .history_get(room_id)
        .await?
        .into_iter()
        .chain(current_vote.into_iter())
        .map(|vote_id| load_from_protocol(volatile.clone(), room_id, vote_id))
        .collect::<Vec<_>>();
    let votes = futures::future::join_all(vote_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, SignalingModuleError>>()?;
    Ok(LegalVoteState { votes })
}
