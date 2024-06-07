// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::{Duration, Instant};

/// This is a poor person's expiring data that tries to emulate the `set_ex`
/// functionality of redis. In order to perform properly, this would have to be
/// cleaned up on a regular basis, because for now it just keeps the value and
/// returns `none` from `value()` if the time expired.
#[derive(Debug, Clone)]
pub struct ExpiringData<T: Clone> {
    value: Option<T>,
    expiry: Option<Instant>,
}

impl<T: Clone> ExpiringData<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            expiry: None,
        }
    }

    pub fn new_ex(value: T, expires_after: Duration) -> Self {
        Self {
            value: Some(value),
            expiry: Instant::now().checked_add(expires_after),
        }
    }

    pub fn value(&self) -> Option<&T> {
        if !self.is_expired() {
            self.value.as_ref()
        } else {
            None
        }
    }

    pub fn take(self) -> Option<T> {
        if !self.is_expired() {
            self.value
        } else {
            None
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expiry.is_some_and(|e| Instant::now() > e)
    }

    pub fn set_expiry(&mut self, expires_after: Duration) {
        self.expiry = Instant::now().checked_add(expires_after);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::ExpiringData;

    #[test]
    fn expiring_data() {
        let expires_after = Duration::from_millis(3);

        let expiring = ExpiringData::new_ex(5, expires_after);
        let nonexpiring = ExpiringData::new(5);

        assert_eq!(expiring.value(), Some(&5));
        assert_eq!(nonexpiring.value(), Some(&5));

        std::thread::sleep(Duration::from_millis(5));

        assert_eq!(expiring.value(), None);
        assert_eq!(nonexpiring.value(), Some(&5));
    }
}
