// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_signaling_core::{control, ModuleContext, SignalingModule};
use opentalk_types_signaling::NamespacedEvent;

pub mod breakout;
pub mod echo;
pub mod moderation;

pub trait ModuleContextExt {
    fn exchange_publish_control(
        &mut self,
        routing_key: String,
        message: control::exchange::Message,
    );
}

impl<M> ModuleContextExt for ModuleContext<'_, M>
where
    M: SignalingModule,
{
    /// Queue a outgoing control message
    ///
    /// Used in modules which control some behavior in the control module/runner
    fn exchange_publish_control(
        &mut self,
        routing_key: String,
        message: control::exchange::Message,
    ) {
        self.exchange_publish_any(
            routing_key,
            NamespacedEvent {
                module: control::MODULE_ID,
                timestamp: self.timestamp(),
                payload: message,
            },
        );
    }
}
