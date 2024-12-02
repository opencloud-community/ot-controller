// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;
use serde::{Deserialize, Serialize};

use super::{Event, EventException, ExternalUser, RegisteredUser, UnregisteredUser};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RegisteredEventInvite::example_data()))
)]
pub struct RegisteredEventInvite {
    pub invitee: RegisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for RegisteredEventInvite {
    fn example_data() -> Self {
        Self {
            invitee: RegisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UnregisteredEventInvite::example_data()))
)]
pub struct UnregisteredEventInvite {
    pub invitee: UnregisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for UnregisteredEventInvite {
    fn example_data() -> Self {
        Self {
            invitee: UnregisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ExternalEventInvite::example_data()))
)]
pub struct ExternalEventInvite {
    pub invitee: ExternalUser,
    pub event: Event,
    pub inviter: RegisteredUser,
    pub invite_code: String,
}

impl ExampleData for ExternalEventInvite {
    fn example_data() -> Self {
        Self {
            invitee: ExternalUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
            invite_code: example_invite_code(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RegisteredEventUpdate::example_data()))
)]
pub struct RegisteredEventUpdate {
    pub invitee: RegisteredUser,
    pub event: Event,
    pub event_exception: Option<EventException>,
    pub inviter: RegisteredUser,
}

impl ExampleData for RegisteredEventUpdate {
    fn example_data() -> Self {
        Self {
            invitee: RegisteredUser::example_data(),
            event: Event::example_data(),
            event_exception: Some(EventException::example_data()),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UnregisteredEventUpdate::example_data()))
)]
pub struct UnregisteredEventUpdate {
    pub invitee: UnregisteredUser,
    pub event: Event,
    pub event_exception: Option<EventException>,
    pub inviter: RegisteredUser,
}

impl ExampleData for UnregisteredEventUpdate {
    fn example_data() -> Self {
        Self {
            invitee: UnregisteredUser::example_data(),
            event: Event::example_data(),
            event_exception: Some(EventException::example_data()),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ExternalEventUpdate::example_data()))
)]
pub struct ExternalEventUpdate {
    pub invitee: ExternalUser,
    pub event: Event,
    pub event_exception: Option<EventException>,
    pub inviter: RegisteredUser,
    pub invite_code: String,
}

impl ExampleData for ExternalEventUpdate {
    fn example_data() -> Self {
        Self {
            invitee: ExternalUser::example_data(),
            event: Event::example_data(),
            event_exception: Some(EventException::example_data()),
            inviter: example_inviter(),
            invite_code: example_invite_code(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RegisteredEventCancellation::example_data()))
)]
pub struct RegisteredEventCancellation {
    pub invitee: RegisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for RegisteredEventCancellation {
    fn example_data() -> Self {
        Self {
            invitee: RegisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UnregisteredEventCancellation::example_data()))
)]
pub struct UnregisteredEventCancellation {
    pub invitee: UnregisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for UnregisteredEventCancellation {
    fn example_data() -> Self {
        Self {
            invitee: UnregisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ExternalEventCancellation::example_data()))
)]
pub struct ExternalEventCancellation {
    pub invitee: ExternalUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for ExternalEventCancellation {
    fn example_data() -> Self {
        Self {
            invitee: ExternalUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RegisteredEventUninvite::example_data()))
)]
pub struct RegisteredEventUninvite {
    pub invitee: RegisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for RegisteredEventUninvite {
    fn example_data() -> Self {
        Self {
            invitee: RegisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UnregisteredEventUninvite::example_data()))
)]
pub struct UnregisteredEventUninvite {
    pub invitee: UnregisteredUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for UnregisteredEventUninvite {
    fn example_data() -> Self {
        Self {
            invitee: UnregisteredUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ExternalEventUninvite::example_data()))
)]
pub struct ExternalEventUninvite {
    pub invitee: ExternalUser,
    pub event: Event,
    pub inviter: RegisteredUser,
}

impl ExampleData for ExternalEventUninvite {
    fn example_data() -> Self {
        Self {
            invitee: ExternalUser::example_data(),
            event: Event::example_data(),
            inviter: example_inviter(),
        }
    }
}

fn example_inviter() -> RegisteredUser {
    RegisteredUser {
        email: "dave.dunn@example.com".into(),
        title: "".parse().expect("valid user title"),
        first_name: "Dave".to_string(),
        last_name: "Dunn".to_string(),
        language: "en".parse().expect("valid language"),
    }
}

fn example_invite_code() -> String {
    "5b5f9cf0-86a8-4e5a-bfac-b05c34c8a20b".to_string()
}
