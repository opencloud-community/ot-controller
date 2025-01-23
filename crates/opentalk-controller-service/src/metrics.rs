// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides some metrics functions and the like.

use opentalk_mail_worker_protocol::MailTask;
use opentelemetry::{
    metrics::{Counter, Histogram},
    Key, KeyValue,
};

const MAIL_TASK_KIND: Key = Key::from_static_str("mail_task_kind");

/// TODO(WR)
#[derive(Debug)]
pub struct EndpointMetrics {
    /// TODO(WR)
    pub request_durations: Histogram<f64>,
    /// TODO(WR)
    pub response_sizes: Histogram<u64>,
    /// TODO(WR)
    pub issued_email_tasks_count: Counter<u64>,
}

impl EndpointMetrics {
    /// TODO(WR)
    pub fn increment_issued_email_tasks_count(&self, mail_task: &MailTask) {
        self.issued_email_tasks_count
            .add(1, &[KeyValue::new(MAIL_TASK_KIND, mail_task.as_kind_str())]);
    }
}
