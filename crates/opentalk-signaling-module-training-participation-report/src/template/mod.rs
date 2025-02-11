// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod checkpoint;
mod report_template_parameter;

pub(crate) use checkpoint::Checkpoint;
pub(crate) use report_template_parameter::ReportTemplateParameter;

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use chrono_tz::Europe::Berlin;
    use opentalk_types_common::users::DisplayName;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{Checkpoint, ReportTemplateParameter};

    pub fn example_small() -> ReportTemplateParameter {
        ReportTemplateParameter {
            title: "OpenTalk introduction training"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: ""
                .parse()
                .expect("value must be parsable as EventDescription"),
            start: "2025-02-18T09:01:23"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            end: "2025-02-18T13:32:02"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: Berlin,
            participants: BTreeMap::from_iter([
                (
                    ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                    Some(
                        "Bob Burton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                ),
                (
                    ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                    Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                ),
            ]),
            checkpoints: vec![
                Checkpoint {
                    timestamp: "2025-02-18T09:22:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([
                        (
                            ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                            "2025-02-18T09:22:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                        (
                            ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                            "2025-02-18T09:22:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                    ]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T11:22:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([
                        (
                            ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                            "2025-02-18T11:25:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                        (
                            ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                            "2025-02-18T11:25:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                    ]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T13:19:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                        "2025-02-18T13:19:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
            ],
        }
    }

    pub fn example_small_json() -> serde_json::Value {
        json!({
          "title": "OpenTalk introduction training",
          "description": "",
          "start": "2025-02-18T09:01:23",
          "end": "2025-02-18T13:32:02",
          "report_timezone": "Europe/Berlin",
          "participants": {
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "Bob Burton",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "Charlie Cooper"
          },
          "checkpoints": [
            {
              "timestamp": "2025-02-18T09:22:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T09:22:00",
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T09:22:00"
              }
            },
            {
              "timestamp": "2025-02-18T11:22:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T11:25:00",
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T11:25:00"
              }
            },
            {
              "timestamp": "2025-02-18T13:19:00",
              "presence": {
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T13:19:00"
              }
           }
          ]
        })
    }

    pub fn example_medium() -> ReportTemplateParameter {
        ReportTemplateParameter {
            title: "OpenTalk introduction training"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: ""
                .parse()
                .expect("value must be parsable as EventDescription"),
            start: "2025-02-18T09:01:23"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            end: "2025-02-19T03:32:02"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: Berlin,
            participants: BTreeMap::from_iter([
                (
                    ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                    Some(
                        "Bob Burton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                ),
                (
                    ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                    Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                ),
            ]),
            checkpoints: vec![
                Checkpoint {
                    timestamp: "2025-02-18T09:22:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([
                        (
                            ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                            "2025-02-18T09:22:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                        (
                            ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                            "2025-02-18T09:22:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                    ]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T11:22:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                        "2025-02-18T11:25:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T13:19:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                        "2025-02-18T13:19:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T15:08:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T17:21:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                        "2025-02-18T17:21:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T19:31:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                        "2025-02-18T19:31:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T21:31:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                        "2025-02-18T21:31:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-18T23:36:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                        "2025-02-18T23:36:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
                Checkpoint {
                    timestamp: "2025-02-19T01:37:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([
                        (
                            ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                            "2025-02-19T01:37:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                        (
                            ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f),
                            "2025-02-19T01:55:00"
                                .parse()
                                .expect("value must be parsable as ReportDateTime"),
                        ),
                    ]),
                },
                Checkpoint {
                    timestamp: "2025-02-19T03:27:00"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    presence: BTreeMap::from_iter([(
                        ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311),
                        "2025-02-19T03:27:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    )]),
                },
            ],
        }
    }

    pub fn example_medium_json() -> serde_json::Value {
        json!({
          "title": "OpenTalk introduction training",
          "description": "",
          "start": "2025-02-18T09:01:23",
          "end": "2025-02-19T03:32:02",
          "report_timezone": "Europe/Berlin",
          "participants": {
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "Bob Burton",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "Charlie Cooper"
          },
          "checkpoints": [
            {
              "timestamp": "2025-02-18T09:22:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T09:22:00",
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T09:22:00"
              }
            },
            {
              "timestamp": "2025-02-18T11:22:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T11:25:00"
              }
            },
            {
              "timestamp": "2025-02-18T13:19:00",
              "presence": {
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T13:19:00"
              }
            },
            {
              "timestamp": "2025-02-18T15:08:00",
              "presence": {}
            },
            {
              "timestamp": "2025-02-18T17:21:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T17:21:00"
              }
            },
            {
              "timestamp": "2025-02-18T19:31:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T19:31:00"
              }
            },
            {
              "timestamp": "2025-02-18T21:31:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T21:31:00"
              }
            },
            {
              "timestamp": "2025-02-18T23:36:00",
              "presence": {
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T23:36:00"
              }
            },
            {
              "timestamp": "2025-02-19T01:37:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-19T01:37:00",
                "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-19T01:55:00"
              }
            },
            {
              "timestamp": "2025-02-19T03:27:00",
              "presence": {
                "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-19T03:27:00"
              }
            }
          ]
        })
    }

    pub fn example_large() -> ReportTemplateParameter {
        let bob_id = ParticipantId::from_u128(0x3ad9e7bf_c0de_4fa8_980e_2a1f55784311);
        let bob: DisplayName = "Bob Burton".parse().unwrap();
        let charlie_id = ParticipantId::from_u128(0x6b9cf256_b8f9_4b20_80e8_5e946118ef0f);
        let charlie: DisplayName = "Charlie Cooper".parse().unwrap();
        let dave_id = ParticipantId::from_u128(0x9945d43a_5123_4bcc_aaf6_4cd4cdd5579c);
        let dave: DisplayName = "Dave Dunn".parse().unwrap();
        let erin_id = ParticipantId::from_u128(0xa39a5488_6d90_4e41_95bd_4cbe17a08899);
        let erin: DisplayName = "Erin Eaton".parse().unwrap();
        let frank_id = ParticipantId::from_u128(0x3cb2a017_fcd6_46bc_8185_a6ac661cd7c2);
        let frank: DisplayName = "Frank Floyd".parse().unwrap();
        let george_id = ParticipantId::from_u128(0x77641ed5_0e14_4fdc_95d4_d1ad2168dcb1);
        let george: DisplayName = "George Garvis".parse().unwrap();
        let hannah_id = ParticipantId::from_u128(0x49351c1d_9c30_4548_966b_a66bc33c0c79);
        let hannah: DisplayName = "Hannah Händl".parse().unwrap();
        let isaac_id = ParticipantId::from_u128(0x70bb4010_6a56_4869_8e3b_fb15d0e6e777);
        let isaac: DisplayName = "Isaac Ivens (Northwind Ltd.)".parse().unwrap();
        let 嬴诗云_id = ParticipantId::from_u128(0xb1e1c7bb_b8f6_4952_bfcf_8db552a24632);
        // Note: this is rendered in the report as "嬴嬴嬴" with the default embedded font,
        // could be changed by using another font.
        let 嬴诗云: DisplayName = "嬴诗云".parse().unwrap();
        let jack_id = ParticipantId::from_u128(0x07a6b5ba_8a9d_40e7_96f0_1c98ff8cf935);
        let jack: DisplayName = "Jack Jilbert".parse().unwrap();
        let karl_id = ParticipantId::from_u128(0x2a59a929_26bb_4f52_b0d4_5f800469aafb);
        let karl: DisplayName = "Karl Keating".parse().unwrap();
        let leann_id = ParticipantId::from_u128(0xd2375f5d_5115_452c_b346_2c9bd9d2c774);
        let leann: DisplayName = "Leann Larn".parse().unwrap();
        let marlene_id = ParticipantId::from_u128(0x573f6d71_8051_4e2a_bf61_9f287642f518);
        let marlene: DisplayName = "Marlene M. Maine".parse().unwrap();
        let neil_id = ParticipantId::from_u128(0xaa5ddb50_e3f0_4235_9f1f_fa2acb3d9b8a);
        let neil: DisplayName = "Neil Neugraten".parse().unwrap();
        let ofelia_id = ParticipantId::from_u128(0xca499540_201d_4f12_8eab_da2ca59aaa72);
        let ofelia: DisplayName = "Ofelia Ollivander".parse().unwrap();
        let patrick_id = ParticipantId::from_u128(0xf1d88851_08c7_4c14_8f5a_46d1f47399d5);
        let patrick: DisplayName = "Patrick Peterson".parse().unwrap();
        let quinton_id = ParticipantId::from_u128(0xb3584f90_4907_46f3_863d_b80df917ec77);
        let quinton: DisplayName = "Quinton Quintana".parse().unwrap();
        let roger_id = ParticipantId::from_u128(0xe3c9bce0_0e3a_4588_bd20_2845368842d4);
        let roger: DisplayName = "Roger Richard".parse().unwrap();
        let sophie_id = ParticipantId::from_u128(0xe35ff04a_5167_4c96_8faf_cf105123d92b);
        let sophie: DisplayName = "Sophie Stanton".parse().unwrap();
        let thalia_id = ParticipantId::from_u128(0xeab201bf_6b5f_490b_8d48_c1cb2c755888);
        let thalia: DisplayName = "Thalia Tyler".parse().unwrap();
        let ulises_id = ParticipantId::from_u128(0xa224bc70_045a_4fe2_ad83_c9be3cf88e34);
        let ulises: DisplayName = "Ulises Underwood".parse().unwrap();
        let valentina_id = ParticipantId::from_u128(0xd2eac94d_a7c9_4faf_8705_43e36e4a035f);
        let valentina: DisplayName = "Valentina Villalobos".parse().unwrap();
        let wallace_id = ParticipantId::from_u128(0xe8c90f50_1c84_4659_985e_2c9c3adfe221);
        let wallace: DisplayName = "Wallace Winters".parse().unwrap();
        let xiomara_id = ParticipantId::from_u128(0x23c72230_c78b_4151_956d_fcbbaad493cb);
        let xiomara: DisplayName = "Xiomara Xiong".parse().unwrap();
        let yousef_id = ParticipantId::from_u128(0x435cd31f_da2c_4410_a695_0140a7141f22);
        let yousef: DisplayName = "Yousef Yu".parse().unwrap();
        let zainab_id = ParticipantId::from_u128(0xbc635c02_78b2_43e1_a980_81272b4c5b38);
        let zainab: DisplayName = "Zainab Zavala".parse().unwrap();

        let timestamp01 = "2025-02-18T09:22:00".parse().unwrap();
        let timestamp02 = "2025-02-18T11:22:00".parse().unwrap();
        let timestamp03 = "2025-02-18T13:19:00".parse().unwrap();
        let timestamp04 = "2025-02-18T15:08:00".parse().unwrap();
        let timestamp05 = "2025-02-18T17:21:00".parse().unwrap();
        let timestamp06 = "2025-02-18T19:31:00".parse().unwrap();
        let timestamp07 = "2025-02-18T21:31:00".parse().unwrap();
        let timestamp08 = "2025-02-18T23:36:00".parse().unwrap();
        let timestamp09 = "2025-02-19T01:37:00".parse().unwrap();
        let timestamp10 = "2025-02-19T03:27:00".parse().unwrap();

        ReportTemplateParameter {
            title: "OpenTalk introduction training"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: ""
                .parse()
                .expect("value must be parsable as EventDescription"),
            start: "2025-02-18T09:01:23"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            end: "2025-02-19T03:32:02"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: Berlin,
            participants: BTreeMap::from_iter([
                (bob_id, Some(bob)),
                (charlie_id, Some(charlie)),
                (dave_id, Some(dave)),
                (erin_id, Some(erin)),
                (frank_id, Some(frank)),
                (george_id, Some(george)),
                (hannah_id, Some(hannah)),
                (isaac_id, Some(isaac)),
                (嬴诗云_id, Some(嬴诗云)),
                (jack_id, Some(jack)),
                (karl_id, Some(karl)),
                (leann_id, Some(leann)),
                (marlene_id, Some(marlene)),
                (neil_id, Some(neil)),
                (ofelia_id, Some(ofelia)),
                (patrick_id, Some(patrick)),
                (quinton_id, Some(quinton)),
                (roger_id, Some(roger)),
                (sophie_id, Some(sophie)),
                (thalia_id, Some(thalia)),
                (ulises_id, Some(ulises)),
                (valentina_id, Some(valentina)),
                (wallace_id, Some(wallace)),
                (xiomara_id, Some(xiomara)),
                (yousef_id, Some(yousef)),
                (zainab_id, Some(zainab)),
            ]),
            checkpoints: vec![
                Checkpoint {
                    timestamp: timestamp01,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp01),
                        (charlie_id, timestamp01),
                        (dave_id, timestamp01),
                        (erin_id, timestamp01),
                        (frank_id, timestamp01),
                        (george_id, timestamp01),
                        (hannah_id, timestamp01),
                        (isaac_id, timestamp01),
                        (嬴诗云_id, timestamp01),
                        (jack_id, timestamp01),
                        (karl_id, timestamp01),
                        (leann_id, timestamp01),
                        (marlene_id, timestamp01),
                        (neil_id, timestamp01),
                        (ofelia_id, timestamp01),
                        (patrick_id, timestamp01),
                        (quinton_id, timestamp01),
                        (roger_id, timestamp01),
                        (sophie_id, timestamp01),
                        (thalia_id, timestamp01),
                        (ulises_id, timestamp01),
                        (valentina_id, timestamp01),
                        (wallace_id, timestamp01),
                        (xiomara_id, timestamp01),
                        (yousef_id, timestamp01),
                        (zainab_id, timestamp01),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp02,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp02),
                        (charlie_id, timestamp02),
                        (dave_id, timestamp02),
                        (erin_id, timestamp02),
                        (frank_id, timestamp02),
                        (george_id, timestamp02),
                        (hannah_id, timestamp02),
                        (isaac_id, timestamp02),
                        (嬴诗云_id, timestamp02),
                        (jack_id, timestamp02),
                        (karl_id, timestamp02),
                        (leann_id, timestamp02),
                        (marlene_id, timestamp02),
                        (neil_id, timestamp02),
                        (ofelia_id, timestamp02),
                        (patrick_id, timestamp02),
                        (quinton_id, timestamp02),
                        (roger_id, timestamp02),
                        (sophie_id, timestamp02),
                        (thalia_id, timestamp02),
                        (ulises_id, timestamp02),
                        (valentina_id, timestamp02),
                        (wallace_id, timestamp02),
                        (xiomara_id, timestamp02),
                        (yousef_id, timestamp02),
                        (zainab_id, timestamp02),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp03,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp03),
                        (charlie_id, timestamp03),
                        (dave_id, timestamp03),
                        (erin_id, timestamp03),
                        (frank_id, timestamp03),
                        (george_id, timestamp03),
                        (hannah_id, timestamp03),
                        (isaac_id, timestamp03),
                        (嬴诗云_id, timestamp03),
                        (jack_id, timestamp03),
                        (karl_id, timestamp03),
                        (leann_id, timestamp03),
                        (marlene_id, timestamp03),
                        (neil_id, timestamp03),
                        (ofelia_id, timestamp03),
                        (patrick_id, timestamp03),
                        (quinton_id, timestamp03),
                        (roger_id, timestamp03),
                        (sophie_id, timestamp03),
                        (thalia_id, timestamp03),
                        (ulises_id, timestamp03),
                        (valentina_id, timestamp03),
                        (wallace_id, timestamp03),
                        (xiomara_id, timestamp03),
                        (yousef_id, timestamp03),
                        (zainab_id, timestamp03),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp04,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp04),
                        (charlie_id, timestamp04),
                        (dave_id, timestamp04),
                        (erin_id, timestamp04),
                        (frank_id, timestamp04),
                        (george_id, timestamp04),
                        (hannah_id, timestamp04),
                        (isaac_id, timestamp04),
                        (嬴诗云_id, timestamp04),
                        (jack_id, timestamp04),
                        (karl_id, timestamp04),
                        (leann_id, timestamp04),
                        (marlene_id, timestamp04),
                        (neil_id, timestamp04),
                        (ofelia_id, timestamp04),
                        (patrick_id, timestamp04),
                        (quinton_id, timestamp04),
                        (roger_id, timestamp04),
                        (sophie_id, timestamp04),
                        (thalia_id, timestamp04),
                        (ulises_id, timestamp04),
                        (valentina_id, timestamp04),
                        (wallace_id, timestamp04),
                        (xiomara_id, timestamp04),
                        (yousef_id, timestamp04),
                        (zainab_id, timestamp04),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp05,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp05),
                        (charlie_id, timestamp05),
                        (dave_id, timestamp05),
                        (erin_id, timestamp05),
                        (frank_id, timestamp05),
                        (george_id, timestamp05),
                        (hannah_id, timestamp05),
                        (isaac_id, timestamp05),
                        (嬴诗云_id, timestamp05),
                        (jack_id, timestamp05),
                        (karl_id, timestamp05),
                        (leann_id, timestamp05),
                        (marlene_id, timestamp05),
                        (neil_id, timestamp05),
                        (ofelia_id, timestamp05),
                        (patrick_id, timestamp05),
                        (quinton_id, timestamp05),
                        (roger_id, timestamp05),
                        (sophie_id, timestamp05),
                        (thalia_id, timestamp05),
                        (ulises_id, timestamp05),
                        (valentina_id, timestamp05),
                        (wallace_id, timestamp05),
                        (xiomara_id, timestamp05),
                        (yousef_id, timestamp05),
                        (zainab_id, timestamp05),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp06,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp06),
                        (charlie_id, timestamp06),
                        (dave_id, timestamp06),
                        (erin_id, timestamp06),
                        (frank_id, timestamp06),
                        (george_id, timestamp06),
                        (hannah_id, timestamp06),
                        (isaac_id, timestamp06),
                        (嬴诗云_id, timestamp06),
                        (jack_id, timestamp06),
                        (karl_id, timestamp06),
                        (leann_id, timestamp06),
                        (marlene_id, timestamp06),
                        (neil_id, timestamp06),
                        (ofelia_id, timestamp06),
                        (patrick_id, timestamp06),
                        (quinton_id, timestamp06),
                        (roger_id, timestamp06),
                        (sophie_id, timestamp06),
                        (thalia_id, timestamp06),
                        (ulises_id, timestamp06),
                        (valentina_id, timestamp06),
                        (wallace_id, timestamp06),
                        (xiomara_id, timestamp06),
                        (yousef_id, timestamp06),
                        (zainab_id, timestamp06),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp07,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp07),
                        (charlie_id, timestamp07),
                        (dave_id, timestamp07),
                        (erin_id, timestamp07),
                        (frank_id, timestamp07),
                        (george_id, timestamp07),
                        (hannah_id, timestamp07),
                        (isaac_id, timestamp07),
                        (嬴诗云_id, timestamp07),
                        (jack_id, timestamp07),
                        (karl_id, timestamp07),
                        (leann_id, timestamp07),
                        (marlene_id, timestamp07),
                        (neil_id, timestamp07),
                        (ofelia_id, timestamp07),
                        (patrick_id, timestamp07),
                        (quinton_id, timestamp07),
                        (roger_id, timestamp07),
                        (sophie_id, timestamp07),
                        (thalia_id, timestamp07),
                        (ulises_id, timestamp07),
                        (valentina_id, timestamp07),
                        (wallace_id, timestamp07),
                        (xiomara_id, timestamp07),
                        (yousef_id, timestamp07),
                        (zainab_id, timestamp07),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp08,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp08),
                        (charlie_id, timestamp08),
                        (dave_id, timestamp08),
                        (erin_id, timestamp08),
                        (frank_id, timestamp08),
                        (george_id, timestamp08),
                        (hannah_id, timestamp08),
                        (isaac_id, timestamp08),
                        (嬴诗云_id, timestamp08),
                        (jack_id, timestamp08),
                        (karl_id, timestamp08),
                        (leann_id, timestamp08),
                        (marlene_id, timestamp08),
                        (neil_id, timestamp08),
                        (ofelia_id, timestamp08),
                        (patrick_id, timestamp08),
                        (quinton_id, timestamp08),
                        (roger_id, timestamp08),
                        (sophie_id, timestamp08),
                        (thalia_id, timestamp08),
                        (ulises_id, timestamp08),
                        (valentina_id, timestamp08),
                        (wallace_id, timestamp08),
                        (xiomara_id, timestamp08),
                        (yousef_id, timestamp08),
                        (zainab_id, timestamp08),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp09,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp09),
                        (charlie_id, timestamp09),
                        (dave_id, timestamp09),
                        (erin_id, timestamp09),
                        (frank_id, timestamp09),
                        (george_id, timestamp09),
                        (hannah_id, timestamp09),
                        (isaac_id, timestamp09),
                        (嬴诗云_id, timestamp09),
                        (jack_id, timestamp09),
                        (karl_id, timestamp09),
                        (leann_id, timestamp09),
                        (marlene_id, timestamp09),
                        (neil_id, timestamp09),
                        (ofelia_id, timestamp09),
                        (patrick_id, timestamp09),
                        (quinton_id, timestamp09),
                        (roger_id, timestamp09),
                        (sophie_id, timestamp09),
                        (thalia_id, timestamp09),
                        (ulises_id, timestamp09),
                        (valentina_id, timestamp09),
                        (wallace_id, timestamp09),
                        (xiomara_id, timestamp09),
                        (yousef_id, timestamp09),
                        (zainab_id, timestamp09),
                    ]),
                },
                Checkpoint {
                    timestamp: timestamp10,
                    presence: BTreeMap::from_iter([
                        (bob_id, timestamp10),
                        (charlie_id, timestamp10),
                        (dave_id, timestamp10),
                        (erin_id, timestamp10),
                        (frank_id, timestamp10),
                        (george_id, timestamp10),
                        (hannah_id, timestamp10),
                        (isaac_id, timestamp10),
                        (嬴诗云_id, timestamp10),
                        (jack_id, timestamp10),
                        (karl_id, timestamp10),
                        (leann_id, timestamp10),
                        (marlene_id, timestamp10),
                        (neil_id, timestamp10),
                        (ofelia_id, timestamp10),
                        (patrick_id, timestamp10),
                        (quinton_id, timestamp10),
                        (roger_id, timestamp10),
                        (sophie_id, timestamp10),
                        (thalia_id, timestamp10),
                        (ulises_id, timestamp10),
                        (valentina_id, timestamp10),
                        (wallace_id, timestamp10),
                        (xiomara_id, timestamp10),
                        (yousef_id, timestamp10),
                        (zainab_id, timestamp10),
                    ]),
                },
            ],
        }
    }

    pub fn example_large_json() -> serde_json::Value {
        let checkpoint01 = json!({
          "timestamp": "2025-02-18T09:22:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T09:22:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T09:22:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T09:22:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T09:22:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T09:22:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T09:22:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T09:22:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T09:22:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T09:22:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T09:22:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T09:22:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T09:22:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T09:22:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T09:22:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T09:22:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T09:22:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T09:22:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T09:22:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T09:22:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T09:22:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T09:22:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T09:22:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T09:22:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T09:22:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T09:22:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T09:22:00"
          }
        });
        let checkpoint02 = json!({
          "timestamp": "2025-02-18T11:22:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T11:22:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T11:22:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T11:22:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T11:22:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T11:22:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T11:22:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T11:22:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T11:22:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T11:22:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T11:22:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T11:22:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T11:22:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T11:22:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T11:22:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T11:22:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T11:22:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T11:22:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T11:22:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T11:22:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T11:22:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T11:22:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T11:22:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T11:22:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T11:22:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T11:22:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T11:22:00"
          }
        });
        let checkpoint03 = json!({
          "timestamp": "2025-02-18T13:19:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T13:19:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T13:19:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T13:19:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T13:19:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T13:19:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T13:19:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T13:19:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T13:19:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T13:19:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T13:19:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T13:19:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T13:19:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T13:19:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T13:19:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T13:19:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T13:19:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T13:19:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T13:19:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T13:19:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T13:19:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T13:19:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T13:19:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T13:19:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T13:19:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T13:19:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T13:19:00"
          }
        });
        let checkpoint04 = json!({
              "timestamp": "2025-02-18T15:08:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T15:08:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T15:08:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T15:08:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T15:08:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T15:08:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T15:08:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T15:08:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T15:08:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T15:08:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T15:08:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T15:08:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T15:08:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T15:08:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T15:08:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T15:08:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T15:08:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T15:08:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T15:08:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T15:08:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T15:08:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T15:08:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T15:08:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T15:08:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T15:08:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T15:08:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T15:08:00"
          }
        });
        let checkpoint05 = json!({
              "timestamp": "2025-02-18T17:21:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T17:21:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T17:21:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T17:21:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T17:21:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T17:21:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T17:21:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T17:21:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T17:21:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T17:21:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T17:21:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T17:21:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T17:21:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T17:21:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T17:21:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T17:21:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T17:21:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T17:21:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T17:21:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T17:21:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T17:21:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T17:21:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T17:21:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T17:21:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T17:21:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T17:21:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T17:21:00"
          }
        });
        let checkpoint06 = json!({
              "timestamp": "2025-02-18T19:31:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T19:31:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T19:31:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T19:31:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T19:31:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T19:31:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T19:31:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T19:31:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T19:31:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T19:31:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T19:31:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T19:31:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T19:31:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T19:31:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T19:31:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T19:31:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T19:31:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T19:31:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T19:31:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T19:31:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T19:31:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T19:31:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T19:31:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T19:31:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T19:31:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T19:31:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T19:31:00"
          }
        });
        let checkpoint07 = json!({
          "timestamp": "2025-02-18T21:31:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T21:31:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T21:31:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T21:31:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T21:31:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T21:31:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T21:31:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T21:31:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T21:31:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T21:31:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T21:31:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T21:31:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T21:31:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T21:31:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T21:31:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T21:31:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T21:31:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T21:31:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T21:31:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T21:31:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T21:31:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T21:31:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T21:31:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T21:31:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T21:31:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T21:31:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T21:31:00"
              }
        });
        let checkpoint08 = json!({
          "timestamp": "2025-02-18T23:36:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-18T23:36:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-18T23:36:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-18T23:36:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-18T23:36:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-18T23:36:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-18T23:36:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-18T23:36:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-18T23:36:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-18T23:36:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-18T23:36:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-18T23:36:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-18T23:36:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-18T23:36:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-18T23:36:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-18T23:36:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-18T23:36:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-18T23:36:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-18T23:36:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-18T23:36:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-18T23:36:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-18T23:36:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-18T23:36:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-18T23:36:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-18T23:36:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-18T23:36:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-18T23:36:00"
          }
        });
        let checkpoint09 = json!({
          "timestamp": "2025-02-19T01:37:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-19T01:37:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-19T01:37:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-19T01:37:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-19T01:37:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-19T01:37:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-19T01:37:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-19T01:37:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-19T01:37:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-19T01:37:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-19T01:37:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-19T01:37:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-19T01:37:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-19T01:37:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-19T01:37:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-19T01:37:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-19T01:37:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-19T01:37:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-19T01:37:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-19T01:37:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-19T01:37:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-19T01:37:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-19T01:37:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-19T01:37:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-19T01:37:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-19T01:37:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-19T01:37:00"
          }
        });
        let checkpoint10 = json!({
          "timestamp": "2025-02-19T03:27:00",
          "presence": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "2025-02-19T03:27:00",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "2025-02-19T03:27:00",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "2025-02-19T03:27:00",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "2025-02-19T03:27:00",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "2025-02-19T03:27:00",
            "435cd31f-da2c-4410-a695-0140a7141f22": "2025-02-19T03:27:00",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "2025-02-19T03:27:00",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "2025-02-19T03:27:00",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "2025-02-19T03:27:00",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "2025-02-19T03:27:00",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "2025-02-19T03:27:00",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "2025-02-19T03:27:00",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "2025-02-19T03:27:00",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "2025-02-19T03:27:00",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "2025-02-19T03:27:00",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "2025-02-19T03:27:00",
            "b3584f90-4907-46f3-863d-b80df917ec77": "2025-02-19T03:27:00",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "2025-02-19T03:27:00",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "2025-02-19T03:27:00",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "2025-02-19T03:27:00",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "2025-02-19T03:27:00",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "2025-02-19T03:27:00",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "2025-02-19T03:27:00",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "2025-02-19T03:27:00",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "2025-02-19T03:27:00",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "2025-02-19T03:27:00"
          }
        });
        json!({
          "title": "OpenTalk introduction training",
          "description": "",
          "start": "2025-02-18T09:01:23",
          "end": "2025-02-19T03:32:02",
          "report_timezone": "Europe/Berlin",
          "participants": {
            "07a6b5ba-8a9d-40e7-96f0-1c98ff8cf935": "Jack Jilbert",
            "23c72230-c78b-4151-956d-fcbbaad493cb": "Xiomara Xiong",
            "2a59a929-26bb-4f52-b0d4-5f800469aafb": "Karl Keating",
            "3ad9e7bf-c0de-4fa8-980e-2a1f55784311": "Bob Burton",
            "3cb2a017-fcd6-46bc-8185-a6ac661cd7c2": "Frank Floyd",
            "435cd31f-da2c-4410-a695-0140a7141f22": "Yousef Yu",
            "49351c1d-9c30-4548-966b-a66bc33c0c79": "Hannah Händl",
            "573f6d71-8051-4e2a-bf61-9f287642f518": "Marlene M. Maine",
            "6b9cf256-b8f9-4b20-80e8-5e946118ef0f": "Charlie Cooper",
            "70bb4010-6a56-4869-8e3b-fb15d0e6e777": "Isaac Ivens (Northwind Ltd.)",
            "77641ed5-0e14-4fdc-95d4-d1ad2168dcb1": "George Garvis",
            "9945d43a-5123-4bcc-aaf6-4cd4cdd5579c": "Dave Dunn",
            "a224bc70-045a-4fe2-ad83-c9be3cf88e34": "Ulises Underwood",
            "a39a5488-6d90-4e41-95bd-4cbe17a08899": "Erin Eaton",
            "aa5ddb50-e3f0-4235-9f1f-fa2acb3d9b8a": "Neil Neugraten",
            "b1e1c7bb-b8f6-4952-bfcf-8db552a24632": "嬴诗云",
            "b3584f90-4907-46f3-863d-b80df917ec77": "Quinton Quintana",
            "bc635c02-78b2-43e1-a980-81272b4c5b38": "Zainab Zavala",
            "ca499540-201d-4f12-8eab-da2ca59aaa72": "Ofelia Ollivander",
            "d2375f5d-5115-452c-b346-2c9bd9d2c774": "Leann Larn",
            "d2eac94d-a7c9-4faf-8705-43e36e4a035f": "Valentina Villalobos",
            "e35ff04a-5167-4c96-8faf-cf105123d92b": "Sophie Stanton",
            "e3c9bce0-0e3a-4588-bd20-2845368842d4": "Roger Richard",
            "e8c90f50-1c84-4659-985e-2c9c3adfe221": "Wallace Winters",
            "eab201bf-6b5f-490b-8d48-c1cb2c755888": "Thalia Tyler",
            "f1d88851-08c7-4c14-8f5a-46d1f47399d5": "Patrick Peterson"
          },
          "checkpoints": [
            checkpoint01,
            checkpoint02,
            checkpoint03,
            checkpoint04,
            checkpoint05,
            checkpoint06,
            checkpoint07,
            checkpoint08,
            checkpoint09,
            checkpoint10,
          ]
        })
    }

    #[test]
    fn serialize_example_small() {
        assert_eq!(json!(example_small()), example_small_json());
    }

    #[test]
    fn deserialize_example_small() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_small_json())
                .expect("value must be deserializable"),
            example_small()
        );
    }

    #[test]
    fn serialize_example_medium() {
        assert_eq!(json!(example_medium()), example_medium_json());
    }

    #[test]
    fn deserialize_example_medium() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_medium_json())
                .expect("value must be deserializable"),
            example_medium()
        );
    }

    #[test]
    fn serialize_example_large() {
        assert_eq!(json!(example_large()), example_large_json());
    }

    #[test]
    fn deserialize_example_large() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_large_json())
                .expect("value must be deserializable"),
            example_large()
        );
    }
}
