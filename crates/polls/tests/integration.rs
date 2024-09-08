// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, time::Duration};

use opentalk_polls::*;
use opentalk_signaling_core::module_tester::{ModuleTester, WsMessageOutgoing};
use opentalk_test_util::*;
use opentalk_types::signaling::polls::event::PollsEvent;
use opentalk_types_signaling_polls::{
    command::{Choices, PollsCommand, Start, Vote},
    event::{Error, Started},
    Choice, ChoiceId, Item, Results,
};
use pretty_assertions::assert_eq;
use serial_test::serial;

async fn start_poll(
    module_tester: &mut ModuleTester<Polls>,
    live: bool,
    multiple_choice: bool,
) -> Started {
    let start = PollsCommand::Start(Start {
        topic: "polling".into(),
        live,
        multiple_choice,
        choices: vec!["yes".into(), "no".into(), "maybe".into()],
        duration: Duration::from_secs(2),
    });

    module_tester
        .send_ws_message(&USER_1.participant_id, start)
        .unwrap();

    let started1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let started2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(started1, started2);

    if let WsMessageOutgoing::Module(PollsEvent::Started(Started {
        id,
        topic,
        live: live_from_event,
        multiple_choice: multiple_choice_from_event,
        choices,
        duration,
    })) = started1
    {
        assert_eq!(topic, "polling");
        assert_eq!(live_from_event, live);
        assert_eq!(multiple_choice_from_event, multiple_choice);
        assert_eq!(
            choices,
            &[
                Choice {
                    id: ChoiceId::from(0),
                    content: "yes".into()
                },
                Choice {
                    id: ChoiceId::from(1),
                    content: "no".into()
                },
                Choice {
                    id: ChoiceId::from(2),
                    content: "maybe".into()
                }
            ]
        );
        assert_eq!(duration.as_millis(), 2000);

        Started {
            id,
            topic,
            live,
            multiple_choice,
            choices,
            duration,
        }
    } else {
        panic!("unexpected {started1:?}")
    }
}

#[actix_rt::test]
#[serial]
async fn full_poll_with_2sec_duration() {
    let test_ctx = TestContext::default().await;

    let (mut module_tester, _user1, _user2) = common::setup_users::<Polls>(&test_ctx, ()).await;

    let started = start_poll(&mut module_tester, true, true).await;

    // User 1 vote: "yes"

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(0)]),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // User 2 vote: "no" (via choice_id field)

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from(1),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // User 2 vote again: "maybe"

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(2)]),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 1,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // User 2 vote again: "yes" and "no"

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(0), ChoiceId::from(1)]),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 2,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // User 2 vote again: invalid choice -> fails

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(3)]),
                },
            }),
        )
        .unwrap();

    let error = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(PollsEvent::Error(Error::InvalidChoiceId)) = error {
        // OK
    } else {
        panic!("unexpected {error:?}")
    }

    // User 2 vote again: `None`

    module_tester
        .send_ws_message(
            &USER_2.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::new(),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // Poll expired, getting results in `Done` event

    let done1 = module_tester
        .receive_ws_message_override_timeout(&USER_1.participant_id, Duration::from_secs(3))
        .await
        .unwrap();

    let done2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(done1, done2);

    if let WsMessageOutgoing::Module(PollsEvent::Done(Results { id, results })) = &done1 {
        assert_eq!(*id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {done1:?}")
    }

    module_tester.shutdown().await.unwrap()
}

#[actix_rt::test]
#[serial]
async fn poll_with_invalid_multiple_choice() {
    let test_ctx = TestContext::default().await;

    let (mut module_tester, _user1, _user2) = common::setup_users::<Polls>(&test_ctx, ()).await;

    let started = start_poll(&mut module_tester, true, false).await;

    // User 1 vote: "yes"

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(0)]),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // User 1 vote: "yes" and "no" (invalid choice) -> fails

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(0), ChoiceId::from(1)]),
                },
            }),
        )
        .unwrap();

    let error = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    if let WsMessageOutgoing::Module(PollsEvent::Error(Error::MultipleChoicesNotAllowed)) = error {
        // OK
    } else {
        panic!("unexpected {error:?}")
    }

    // User 1 vote: "no"

    module_tester
        .send_ws_message(
            &USER_1.participant_id,
            PollsCommand::Vote(Vote {
                poll_id: started.id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from_iter(vec![ChoiceId::from(1)]),
                },
            }),
        )
        .unwrap();

    let update1 = module_tester
        .receive_ws_message(&USER_1.participant_id)
        .await
        .unwrap();

    let update2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(update1, update2);

    if let WsMessageOutgoing::Module(PollsEvent::LiveUpdate(Results { id, results })) = update1 {
        assert_eq!(id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {update1:?}")
    }

    // Poll expired, getting results in `Done` event

    let done1 = module_tester
        .receive_ws_message_override_timeout(&USER_1.participant_id, Duration::from_secs(3))
        .await
        .unwrap();

    let done2 = module_tester
        .receive_ws_message(&USER_2.participant_id)
        .await
        .unwrap();

    assert_eq!(done1, done2);

    if let WsMessageOutgoing::Module(PollsEvent::Done(Results { id, results })) = &done1 {
        assert_eq!(*id, started.id);
        assert_eq!(
            results,
            &[
                Item {
                    id: ChoiceId::from(0),
                    count: 0,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 1,
                },
                Item {
                    id: ChoiceId::from(2),
                    count: 0,
                }
            ]
        );
    } else {
        panic!("unexpected {done1:?}")
    }

    module_tester.shutdown().await.unwrap()
}
