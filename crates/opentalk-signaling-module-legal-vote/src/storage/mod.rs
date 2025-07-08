// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod protocol;
mod redis;
mod volatile;

mod legal_vote_storage;

use ::redis::{ErrorKind, FromRedisValue, RedisError, RedisResult, Value};
pub(crate) use legal_vote_storage::{
    LegalVoteAllowTokenStorage, LegalVoteCurrentStorage, LegalVoteHistoryStorage,
    LegalVoteParameterStorage, LegalVoteStorage,
};
pub use protocol::{NewProtocol, Protocol, v1};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VoteStatus {
    Active = 0,
    Complete,
    Unknown,
}

impl FromRedisValue for VoteStatus {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Int(val) = v {
            match val {
                0 => Ok(VoteStatus::Active),
                1 => Ok(VoteStatus::Complete),
                2 => Ok(VoteStatus::Unknown),
                _ => Err(RedisError::from((
                    ErrorKind::TypeError,
                    "Vote status script must return int values between 0 an 2",
                ))),
            }
        } else {
            Err(RedisError::from((
                ErrorKind::TypeError,
                "Vote status script must return int value",
            )))
        }
    }
}

/// Mapping for codes that are returned by the [`VOTE_SCRIPT`]
pub(crate) enum VoteScriptResult {
    // Vote successful
    Success = 0,
    // Vote closed successfully & no more allowed users
    SuccessAutoClose,
    // Provided vote id was not active
    InvalidVoteId,
    // User is not allowed to vote
    Ineligible,
}

impl FromRedisValue for VoteScriptResult {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Int(val) = v {
            match val {
                0 => Ok(VoteScriptResult::Success),
                1 => Ok(VoteScriptResult::SuccessAutoClose),
                2 => Ok(VoteScriptResult::InvalidVoteId),
                3 => Ok(VoteScriptResult::Ineligible),

                _ => Err(RedisError::from((
                    ErrorKind::TypeError,
                    "Vote script must return int value between 0 and 3",
                ))),
            }
        } else {
            Err(RedisError::from((
                ErrorKind::TypeError,
                "Vote script must return int value",
            )))
        }
    }
}

#[cfg(test)]
pub(crate) mod test_common {
    use std::vec;

    use chrono::DateTime;
    use opentalk_signaling_core::SignalingRoomId;
    use opentalk_types_common::users::UserId;
    use opentalk_types_signaling::ParticipantId;
    use opentalk_types_signaling_legal_vote::{
        parameters::Parameters,
        token::Token,
        user_parameters::{AllowedParticipants, Duration, UserParameters},
        vote::{LegalVoteId, VoteKind, VoteOption},
    };
    use pretty_assertions::assert_eq;

    use super::LegalVoteStorage;
    use crate::storage::{VoteStatus, protocol::v1::Vote};

    pub(crate) const ROOM: SignalingRoomId = SignalingRoomId::nil();
    pub(crate) const VOTE: LegalVoteId = LegalVoteId::nil();

    const ALICE_USER: UserId = UserId::from_u128(0xbadcafe);
    const ALICE_PARTICIPANT: ParticipantId = ParticipantId::from_u128(0xbadcafe);
    pub(crate) async fn allow_token(storage: &mut dyn LegalVoteStorage) {
        storage
            .allow_token_set(
                ROOM,
                LegalVoteId::generate(),
                vec![Token::new(1), Token::new(2)],
            )
            .await
            .unwrap()
    }

    fn generate_parameter() -> Parameters {
        Parameters {
            initiator_id: ALICE_PARTICIPANT,
            legal_vote_id: VOTE,
            start_time: DateTime::from_timestamp_millis(1).unwrap(),
            max_votes: 2,
            allowed_users: Some(vec![ALICE_USER]),
            token: Some(Token::generate()),
            inner: UserParameters {
                name: "TestWithOptionalFields".parse().unwrap(),
                kind: VoteKind::RollCall,
                subtitle: Some("A subtitle".parse().unwrap()),
                topic: Some("Yes or No?".parse().unwrap()),
                allowed_participants: AllowedParticipants::try_from(vec![
                    ParticipantId::from_u128(1),
                    ParticipantId::from_u128(2),
                ])
                .unwrap(),
                enable_abstain: false,
                auto_close: false,
                duration: Some(Duration::try_from(5u64).unwrap()),
                create_pdf: true,
                timezone: Some(chrono_tz::CET),
            },
        }
    }

    pub(crate) async fn current_vote(storage: &mut dyn LegalVoteStorage) {
        assert!(storage.current_vote_get(ROOM).await.unwrap().is_none());

        assert!(storage.current_vote_set(ROOM, VOTE).await.unwrap());
        assert_eq!(Some(VOTE), storage.current_vote_get(ROOM).await.unwrap());

        let replacement_vote = LegalVoteId::generate();
        assert!(
            !storage
                .current_vote_set(ROOM, replacement_vote)
                .await
                .unwrap()
        );
        assert_eq!(Some(VOTE), storage.current_vote_get(ROOM).await.unwrap());

        storage.current_vote_delete(ROOM).await.unwrap();
        assert!(storage.current_vote_get(ROOM).await.unwrap().is_none());
    }

    pub(crate) async fn parameter(storage: &mut dyn LegalVoteStorage) {
        let parameter: Parameters = generate_parameter();

        assert!(storage.parameter_get(ROOM, VOTE).await.unwrap().is_none());
        storage.parameter_set(ROOM, VOTE, &parameter).await.unwrap();
        assert_eq!(
            Some(parameter),
            storage.parameter_get(ROOM, VOTE).await.unwrap()
        );

        storage.cleanup_vote(ROOM, VOTE).await.unwrap();
        assert!(storage.parameter_get(ROOM, VOTE).await.unwrap().is_none());
    }

    pub(crate) async fn voting(storage: &mut dyn LegalVoteStorage) {
        assert!(storage.current_vote_get(ROOM).await.unwrap().is_none());
        let parameter: Parameters = generate_parameter();
        assert_eq!(
            VoteStatus::Unknown,
            storage.get_vote_status(ROOM, VOTE).await.unwrap()
        );

        storage.parameter_set(ROOM, VOTE, &parameter).await.unwrap();
        storage.current_vote_set(ROOM, VOTE).await.unwrap();

        storage
            .vote(
                ROOM,
                VOTE,
                Vote {
                    user_info: None,
                    token: parameter.token.unwrap(),
                    option: VoteOption::Yes,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            VoteStatus::Active,
            storage.get_vote_status(ROOM, VOTE).await.unwrap()
        );
    }
}
