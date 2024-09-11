// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ParticipantPair {
    participant_one: ParticipantId,
    participant_two: ParticipantId,
}

impl ParticipantPair {
    pub fn new(participant_one: ParticipantId, participant_two: ParticipantId) -> Self {
        if participant_two < participant_one {
            Self {
                participant_one: participant_two,
                participant_two: participant_one,
            }
        } else {
            Self {
                participant_one,
                participant_two,
            }
        }
    }

    pub fn participant_one(&self) -> ParticipantId {
        self.participant_one
    }

    pub fn participant_two(&self) -> ParticipantId {
        self.participant_two
    }

    pub fn as_tuple(&self) -> (ParticipantId, ParticipantId) {
        (self.participant_one, self.participant_two)
    }
}

impl From<(ParticipantId, ParticipantId)> for ParticipantPair {
    fn from((participant_one, participant_two): (ParticipantId, ParticipantId)) -> Self {
        Self::new(participant_one, participant_two)
    }
}

impl From<ParticipantPair> for (ParticipantId, ParticipantId) {
    fn from(value: ParticipantPair) -> Self {
        value.as_tuple()
    }
}

enum IterState {
    ParticipantOne,
    ParticipantTwo,
    Finished,
}

pub(crate) struct IntoIter {
    state: IterState,
    pair: ParticipantPair,
}

impl IntoIter {
    fn new(pair: ParticipantPair) -> Self {
        Self {
            state: IterState::ParticipantOne,
            pair,
        }
    }
}

impl Iterator for IntoIter {
    type Item = ParticipantId;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IterState::ParticipantOne => {
                self.state = IterState::ParticipantTwo;
                Some(self.pair.participant_one)
            }
            IterState::ParticipantTwo => {
                self.state = IterState::Finished;
                Some(self.pair.participant_two)
            }
            IterState::Finished => None,
        }
    }
}

impl IntoIterator for ParticipantPair {
    type Item = ParticipantId;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
