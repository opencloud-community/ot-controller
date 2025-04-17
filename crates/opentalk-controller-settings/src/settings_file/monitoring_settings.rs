// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::net::IpAddr;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MonitoringSettings {
    #[serde(default = "default_monitoring_port")]
    pub port: u16,
    #[serde(default = "default_monitoring_addr")]
    pub addr: IpAddr,
}

fn default_monitoring_port() -> u16 {
    11411
}

fn default_monitoring_addr() -> IpAddr {
    [0, 0, 0, 0].into()
}
