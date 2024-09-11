// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{any::Any, pin::Pin};

use futures::{Stream, StreamExt};
use opentalk_types_common::modules::ModuleId;

pub type AnyStream = Pin<Box<dyn Stream<Item = (ModuleId, Box<dyn Any + 'static>)>>>;

pub fn any_stream<S>(module_id: ModuleId, stream: S) -> AnyStream
where
    S: Stream + 'static,
{
    Box::pin(
        stream.map(move |item| -> (_, Box<dyn Any + 'static>) {
            (module_id.clone(), Box::new(item))
        }),
    )
}
