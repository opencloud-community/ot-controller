// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub mod data;

use data::ReportData;
pub use error::Error;
use opentalk_types_common::users::{DisplayName, UserId};

mod error;
mod report_data_builder;

use std::{collections::BTreeMap, path::Path};

use chrono_tz::Tz;
use error::ReportGenerationSnafu;
use report_data_builder::Builder;
use snafu::ResultExt as _;

use crate::storage::v1::ProtocolEntry;

const DEFAULT_TEMPLATE: &str = include_str!("legal_vote_report.typ");

pub(crate) fn generate(
    user_names: BTreeMap<UserId, DisplayName>,
    protocol: Vec<ProtocolEntry>,
    timezone: &Tz,
    dump_to_relative_path: &Path,
) -> Result<Vec<u8>, Error> {
    let builder = Builder::new(user_names);
    let report_data = builder.build_report_data(protocol, timezone)?;

    generate_from_template(
        DEFAULT_TEMPLATE.to_string(),
        &report_data,
        dump_to_relative_path,
    )
}

fn generate_from_template(
    template: String,
    parameter: &ReportData,
    dump_to_relative_path: &Path,
) -> Result<Vec<u8>, Error> {
    let dump_to_path = std::env::var("OPENTALK_REPORT_DUMP_PATH")
        .map(|p| Path::new(&p).join(dump_to_relative_path))
        .ok();

    opentalk_report_generation::generate_pdf_report(
        template,
        BTreeMap::from_iter([(
            Path::new("data.json"),
            serde_json::to_string_pretty(parameter)
                .unwrap()
                .into_bytes()
                .into(),
        )]),
        dump_to_path.as_deref(),
    )
    .context(ReportGenerationSnafu)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use insta::assert_snapshot;

    use super::{
        DEFAULT_TEMPLATE,
        data::{
            ReportData,
            report_data::tests::{example_live_roll_call, example_pseudonymous, example_roll_call},
        },
        generate_from_template,
    };
    use crate::{MODULE_ID, report::data::report_data::tests::canceled_live_roll_call};

    fn generate(sample_name: &str, parameter: &ReportData) -> String {
        let pdf = generate_from_template(
            DEFAULT_TEMPLATE.to_string(),
            parameter,
            Path::new(&format!("{MODULE_ID}/{sample_name}")),
        )
        .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn generate_report_live_roll_call() {
        assert_snapshot!(
            generate("live_roll_call", &example_live_roll_call()),
            @r#"
        OpenTalk Vote Report
         Title : Weather Vote

        Subtitle : Another one of these weather votes

        Topic : Is the weather good today?

        Vote kind : Live roll call

        Referendum leader : Alice Adams

        Vote id : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Start : 2025-01-02 03:04:05

        End : 2025-01-02 03:09:05

        Report timezone : Europe/Berlin

        Participant count : 8

        Scheduled duration : 300 s

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 6

        Results
         Vote Count

        Yes 4

        No 2

        Abstain 5

        Recorded votes
         Name Token Vote Timestamp

        Alice Adams aaaaaaaa Yes 2025-01-02 03:04:24

        Bob Burton bbbbbbbb No 2025-01-02 03:04:20

        Charlie Cooper cccccccc No 2025-01-02 03:04:21

        Dave Dunn dddddddd Yes 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Yes 2025-01-02 03:06:00

        George Grump gggggggg Yes 2025-01-02 03:06:00

        Event log
         Name Timestamp Event

        Charlie Cooper 2025-01-02 03:04:18 Reports a screenshare issue
        "#
        );
    }

    #[test]
    fn generate_report_roll_call() {
        assert_snapshot!(
            generate("roll_call", &example_roll_call()),
            @r#"
        OpenTalk Vote Report
         Title : End meeting early

        Subtitle : Should we end today's meeting earlier?

        Vote kind : Roll call

        Referendum leader : Alice Adams

        Vote id : 21ca8797-915a-4255-86d7-23ad0508905f

        Start : 2025-02-09 08:41:50

        End : 2025-02-09 08:42:50

        Report timezone : Europe/Vienna

        Participant count : 4

        Scheduled duration : 60 s

        Abstention : Disallowed

        Automatic close : Disabled

        Vote ended due to : Expired

        Number of votes : 3

        Results
         Vote Count

        Yes 2

        No 1

        Recorded votes
         Name Token Vote Timestamp

        Bob Burton WPFPHL6RH7Y No 2025-02-09 08:41:56

        Alice Adams FmrpkqvtHN8 Yes 2025-02-09 08:42:00

        Dave Dunn 538ks7CrBET Yes 2025-02-09 08:42:35

        Event log
         Name Timestamp Event

        Alice Adams 2025-02-09 08:42:16 Reports a problem : Misunderstood the question :-(

        Charlie Cooper 2025-02-09 08:42:26 User left

        Charlie Cooper 2025-02-09 08:42:28 User joined
        "#
        );
    }

    #[test]
    fn generate_pseudonymous() {
        assert_snapshot!(generate("pseudonymous",&example_pseudonymous()),
        @r#"
        OpenTalk Vote Report
         Title : Example Pseudonymous Vote

        Vote kind : Pseudonymous vote

        Referendum leader : Alice Adams

        Vote id : 6a3525fc-aeef-4d7e-9d76-e41ab2cbe469

        Start : 2025-02-08 12:32:09

        End : 2025-02-08 12:32:22

        Report timezone : Europe/Vienna

        Participant count : 4

        Scheduled duration : 60 s

        Abstention : Allowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 4

        Results
         Vote Count

        Yes 1

        No 2

        Abstain 1

        Recorded votes
         Name Token Vote Timestamp

        Hidden LPwNXJWs7b1 Yes —

        Hidden K5SMSt98f11 No —

        Hidden B1yWM5eWQQi Abstain —

        Hidden 8PCkuJ9NGoY No —

        Event log
         Name Timestamp Event
        "#);
    }

    #[test]
    fn generate_canceled_live_roll_call() {
        assert_snapshot!(
            generate("canceled_live_roll_call", &canceled_live_roll_call()),
            @r#"
        OpenTalk Vote Report
         Title : Weather Vote

        Subtitle : Another one of these weather votes

        Topic : Is the weather good today?

        Vote kind : Live roll call

        Referendum leader : Alice Adams

        Vote id : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Start : 2025-01-02 03:04:05

        End : 2025-01-02 03:09:05

        Report timezone : Europe/Berlin

        Participant count : 8

        Scheduled duration : 300 s

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : Aborted for custom reason: test

        Number of votes : 6

        Recorded votes
         Name Token Vote Timestamp

        Alice Adams aaaaaaaa Yes 2025-01-02 03:04:24

        Bob Burton bbbbbbbb No 2025-01-02 03:04:20

        Charlie Cooper cccccccc No 2025-01-02 03:04:21

        Dave Dunn dddddddd Yes 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Yes 2025-01-02 03:06:00

        George Grump gggggggg Yes 2025-01-02 03:06:00

        Event log
         Name Timestamp Event

        Charlie Cooper 2025-01-02 03:04:18 Reports a screenshare issue
        "#
        );
    }
}
