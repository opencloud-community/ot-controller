// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::ReportsTemplate;

#[derive(Default, Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Reports {
    #[serde(default)]
    pub template: ReportsTemplate,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::settings_file::{Reports, ReportsTemplate};

    #[test]
    fn meeting_report_settings() {
        let toml_settings: Reports = toml::from_str(
            r#"
        url = "http://localhost"
        "#,
        )
        .unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                template: ReportsTemplate::BuiltIn
            }
        );

        let toml_settings: Reports = toml::from_str(
            r#"
        url = "http://localhost"
        template.inline = "lorem ipsum"
        "#,
        )
        .unwrap();
        assert_eq!(
            toml_settings,
            Reports {
                template: ReportsTemplate::Inline("lorem ipsum".to_string())
            }
        );
    }
}
