// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::module_tester::WsMessageOutgoing;
use opentalk_signaling_module_automod as automod;
use opentalk_test_util::{common, TestContext, TestUser, USER_1, USER_2};
use opentalk_types_signaling::{ParticipantId, Role};
use opentalk_types_signaling_automod::{
    command::{AutomodCommand, Edit, Select, Start, Yield},
    config::{Parameter, SelectionStrategy},
    event::{AutomodEvent, Error, RemainingUpdated, SpeakerUpdated, StoppedReason},
};
use opentalk_types_signaling_control::event::ControlEvent;
use pretty_assertions::assert_eq;
use serial_test::serial;

#[actix_rt::test]
#[serial]
async fn reject_start_empty_allow_or_playlist() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                allow_list: None,
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Error(Error::InvalidSelection)) = answer {
        // yay
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn reject_start_invalid_allow_list() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add the invalid user
                allow_list: Some(vec![
                    ParticipantId::from_u128(123457890),
                    ParticipantId::from_u128(978123987234),
                ]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Error(Error::InvalidSelection)) = answer {
        // yay
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn reject_start_invalid_allow_list_with_some_correct() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add the invalid user
                allow_list: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    ParticipantId::from_u128(123457890),
                    ParticipantId::from_u128(978123987234),
                ]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Error(Error::InvalidSelection)) = answer {
        // yay
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn reject_start_if_session_already_running() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add valid users
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Started(_)) = answer {
        // ok
    } else {
        panic!()
    }

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add valid users
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Error(Error::SessionAlreadyRunning)) = answer {
        // yay
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn accept_valid_edit() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add valid users
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Started(_)) = answer {
        // ok
    } else {
        panic!()
    }

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Edit(Edit {
                allow_list: Some(vec![USER_1.participant_id]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::RemainingUpdated(RemainingUpdated {
        remaining,
    })) = answer
    {
        assert_eq!(remaining, &[USER_1.participant_id]);
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn reject_invalid_edit() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Random,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: false,
                },
                // Add valid users
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Started(_)) = answer {
        // ok
    } else {
        panic!()
    }

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Edit(Edit {
                allow_list: Some(vec![
                    USER_1.participant_id,
                    ParticipantId::from_u128(978653421),
                ]),
                playlist: None,
            }),
        )
        .unwrap();

    let answer = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::Error(Error::InvalidSelection)) = answer {
        // yay
    } else {
        panic!()
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn auto_append_playlist() {
    auto_append(SelectionStrategy::Playlist).await
}

#[actix_rt::test]
#[serial]
async fn auto_append_random() {
    auto_append(SelectionStrategy::Random).await
}

#[actix_rt::test]
#[serial]
async fn auto_append_nomination() {
    auto_append(SelectionStrategy::Nomination).await
}

#[actix_rt::test]
#[serial]
async fn auto_append_none() {
    auto_append(SelectionStrategy::None).await
}

async fn auto_append(selection_strategy: SelectionStrategy) {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: true,
                },
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: Some(vec![USER_1.participant_id, USER_2.participant_id]),
            }),
        )
        .unwrap();

    let started1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if !matches!(
        started1,
        WsMessageOutgoing::Module(AutomodEvent::Started(_))
    ) {
        panic!("expected start message, got {:?}", started1);
    }

    let started2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(started1, started2);

    // Create and join with user3

    let user3 = test_ctx.db_ctx.create_test_user(3, vec![]).await.unwrap();

    module_tester
        .join_user(
            ParticipantId::from_u128(3),
            user3.clone(),
            Role::User,
            &USER_1.display_name(),
            (),
        )
        .await
        .unwrap();

    let control_user3_joined = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    assert!(matches!(
        control_user3_joined,
        WsMessageOutgoing::Control(ControlEvent::Joined(_))
    ));

    let remaining = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(AutomodEvent::RemainingUpdated(RemainingUpdated {
        remaining,
    })) = remaining
    {
        match selection_strategy {
            SelectionStrategy::None | SelectionStrategy::Random | SelectionStrategy::Nomination => {
                assert!(remaining.contains(&USER_1.participant_id));
                assert!(remaining.contains(&USER_2.participant_id));
                assert!(remaining.contains(&ParticipantId::from_u128(3)));
            }

            SelectionStrategy::Playlist => {
                assert_eq!(
                    remaining,
                    vec![
                        USER_1.participant_id,
                        USER_2.participant_id,
                        ParticipantId::from_u128(3)
                    ]
                )
            }
        }
    } else {
        panic!("Expected `RemainingUpdated`")
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn full_run_playlist() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    const USER_3: TestUser = TestUser {
        n: 3,
        participant_id: ParticipantId::from_u128(3),
        name: "user3",
    };

    // Create and join with user3
    let user3 = test_ctx.db_ctx.create_test_user(3, vec![]).await.unwrap();

    module_tester
        .join_user(
            ParticipantId::from_u128(3),
            user3.clone(),
            Role::User,
            &USER_1.display_name(),
            (),
        )
        .await
        .unwrap();

    for user in [&USER_1, &USER_2] {
        let control_user3_joined = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert!(matches!(
            control_user3_joined,
            WsMessageOutgoing::Control(ControlEvent::Joined(_))
        ));
    }

    let control_user3_join_success = module_tester
        .receive_ws_message(&USER_3.participant_id)
        .await
        .unwrap();
    assert!(matches!(
        control_user3_join_success,
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(_))
    ));

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Playlist,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: true,
                },
                allow_list: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    USER_3.participant_id,
                ]),
                playlist: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    USER_3.participant_id,
                ]),
            }),
        )
        .unwrap();

    let started1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if !matches!(
        started1,
        WsMessageOutgoing::Module(AutomodEvent::Started(_))
    ) {
        panic!("expected start message, got {:?}", started1);
    }

    let started2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(started2, started1);

    let started3 = module_tester
        .receive_ws_message(&USER_3.participant_id)
        .await
        .unwrap();

    assert_eq!(started3, started1);

    module_tester
        .send_ws_message(&USER_1.participant_id, AutomodCommand::Select(Select::Next))
        .unwrap();

    for user in [&USER_1, &USER_2, &USER_3] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                speaker: Some(USER_1.participant_id),
                history: Some(vec![USER_1.participant_id]),
                remaining: Some(vec![USER_2.participant_id, USER_3.participant_id]),
            }))
        );
    }

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Yield(Yield { next: None }),
        )
        .unwrap();

    for user in [&USER_1, &USER_2, &USER_3] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                speaker: Some(USER_2.participant_id),
                history: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                remaining: Some(vec![USER_3.participant_id]),
            }))
        );
    }

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            AutomodCommand::Yield(Yield { next: None }),
        )
        .unwrap();

    for user in [&USER_1, &USER_2, &USER_3] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                speaker: Some(USER_3.participant_id),
                history: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    USER_3.participant_id
                ]),
                remaining: Some(vec![]),
            }))
        );
    }

    module_tester
        .send_ws_message(
            &USER_3.participant_id,
            AutomodCommand::Yield(Yield { next: None }),
        )
        .unwrap();

    for user in [&USER_1, &USER_2, &USER_3] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::Stopped(StoppedReason::SessionFinished))
        );
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn on_leaving_sends_remaning_update() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    const USER_3: TestUser = TestUser {
        n: 3,
        participant_id: ParticipantId::from_u128(3),
        name: "user3",
    };

    // Create and join with user3
    let user3 = test_ctx.db_ctx.create_test_user(3, vec![]).await.unwrap();

    module_tester
        .join_user(
            ParticipantId::from_u128(3),
            user3.clone(),
            Role::User,
            &USER_1.display_name(),
            (),
        )
        .await
        .unwrap();

    for user in [&USER_1, &USER_2] {
        let control_user3_joined = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert!(matches!(
            control_user3_joined,
            WsMessageOutgoing::Control(ControlEvent::Joined(_))
        ));
    }

    let control_user3_join_success = module_tester
        .receive_ws_message(&USER_3.participant_id)
        .await
        .unwrap();
    assert!(matches!(
        control_user3_join_success,
        WsMessageOutgoing::Control(ControlEvent::JoinSuccess(_))
    ));

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Playlist,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: true,
                },
                allow_list: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    USER_3.participant_id,
                ]),
                playlist: Some(vec![
                    USER_1.participant_id,
                    USER_2.participant_id,
                    USER_3.participant_id,
                ]),
            }),
        )
        .unwrap();

    let started1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if !matches!(
        started1,
        WsMessageOutgoing::Module(AutomodEvent::Started(_))
    ) {
        panic!("expected start message, got {:?}", started1);
    }

    let started2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(started2, started1);

    let started3 = module_tester
        .receive_ws_message(&USER_3.participant_id)
        .await
        .unwrap();

    assert_eq!(started3, started1);

    module_tester.leave(&USER_3.participant_id).await.unwrap();

    for user in [&USER_1, &USER_2] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::RemainingUpdated(RemainingUpdated {
                remaining: vec![USER_1.participant_id, USER_2.participant_id],
            }))
        );
    }

    module_tester.shutdown().await.unwrap();
}

#[actix_rt::test]
#[serial]
async fn skipping_last_speaker_finishes_the_session() {
    let test_ctx = TestContext::default().await;
    let (mut module_tester, _user1, _user2) =
        common::setup_users::<automod::Automod>(&test_ctx, ()).await;

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            AutomodCommand::Start(Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::Playlist,
                    show_list: true,
                    consider_hand_raise: false,
                    time_limit: None,
                    allow_double_selection: false,
                    animation_on_random: true,
                    auto_append_on_join: true,
                },
                allow_list: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                playlist: Some(vec![USER_1.participant_id, USER_2.participant_id]),
            }),
        )
        .unwrap();

    let started1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if !matches!(
        started1,
        WsMessageOutgoing::Module(AutomodEvent::Started(_))
    ) {
        panic!("expected start message, got {:?}", started1);
    }

    let started2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(started2, started1);

    module_tester
        .send_ws_message(&USER_1.participant_id, AutomodCommand::Select(Select::Next))
        .unwrap();

    for user in [&USER_1, &USER_2] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                speaker: Some(USER_1.participant_id),
                history: Some(vec![USER_1.participant_id]),
                remaining: Some(vec![USER_2.participant_id]),
            }))
        );
    }

    module_tester
        .send_ws_message(&USER_1.participant_id, AutomodCommand::Select(Select::Next))
        .unwrap();

    for user in [&USER_1, &USER_2] {
        let update = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            update,
            WsMessageOutgoing::Module(AutomodEvent::SpeakerUpdated(SpeakerUpdated {
                speaker: Some(USER_2.participant_id),
                history: Some(vec![USER_1.participant_id, USER_2.participant_id]),
                remaining: Some(vec![]),
            }))
        );
    }

    module_tester
        .send_ws_message(&USER_1.participant_id, AutomodCommand::Select(Select::Next))
        .unwrap();

    for user in [&USER_1, &USER_2] {
        let stopped = module_tester
            .receive_ws_message(&user.participant_id)
            .await
            .unwrap();

        assert_eq!(
            stopped,
            WsMessageOutgoing::Module(AutomodEvent::Stopped(StoppedReason::SessionFinished))
        );
    }

    module_tester.shutdown().await.unwrap();
}
