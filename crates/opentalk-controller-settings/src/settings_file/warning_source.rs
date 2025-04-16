// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[derive(Debug, Clone)]
pub(super) struct WarningSource<T: Clone>(T);

impl<T: Clone> WarningSource<T> {
    pub(super) fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> config::Source for WarningSource<T>
where
    T: config::Source + Send + Sync + Clone + 'static,
{
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        let values = self.0.collect()?;
        if !values.is_empty() {
            use owo_colors::OwoColorize as _;

            anstream::eprintln!(
                "{}: The following environment variables have been deprecated and \
                will not work in a future release. Please change them as suggested below:",
                "DEPRECATION WARNING".yellow().bold(),
            );

            for key in values.keys() {
                let env_var = key.replace('.', "__").to_uppercase();
                anstream::eprintln!(
                    "{}: rename environment variable {} to {}",
                    "DEPRECATION WARNING".yellow().bold(),
                    format!("K3K_CTRL_{}", env_var).yellow(),
                    format!("OPENTALK_CTRL_{}", env_var).green().bold(),
                );
            }
        }

        Ok(values)
    }
}
