// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub trait VolatileStorage {}

pub struct VolatileStaticMemoryStorage;

impl VolatileStorage for VolatileStaticMemoryStorage {}
