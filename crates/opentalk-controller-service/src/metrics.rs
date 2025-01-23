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

/// Metrics belonging to endpoints
#[derive(Debug)]
pub struct EndpointMetrics {
    /// A histogram for the request durations
    pub request_durations: Histogram<f64>,
    /// A histogram for the response sizes
    pub response_sizes: Histogram<u64>,
    /// A counter for the issued email tasks
    pub issued_email_tasks_count: Counter<u64>,
}

impl EndpointMetrics {
    /// Increment the number of issued email tasks
    pub fn increment_issued_email_tasks_count(&self, mail_task: &MailTask) {
        self.issued_email_tasks_count
            .add(1, &[KeyValue::new(MAIL_TASK_KIND, mail_task.as_kind_str())]);
    }
}
