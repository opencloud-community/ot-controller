// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use scoped_futures::ScopedBoxFuture;

use crate::{Error, Inventory, Result};

/// Execute the given function inside a transaction.
pub async fn transaction<'a, F, R, E>(inventory: &mut dyn Inventory, callback: F) -> Result<R, E>
where
    F: for<'r> FnOnce(&'r mut dyn Inventory) -> ScopedBoxFuture<'a, 'r, Result<R, E>> + Send + 'a,
    R: Send,
    E: From<Error> + Send,
{
    inventory.begin_transaction().await?;
    match callback(&mut *inventory).await {
        Ok(value) => {
            inventory.commit_transaction().await?;
            Ok(value)
        }
        Err(user_error) => match inventory.rollback_transaction().await {
            Ok(()) => Err(user_error),
            Err(Error::BrokenTransactionManager) => Err(user_error),
            Err(rollback_error) => Err(rollback_error.into()),
        },
    }
}
