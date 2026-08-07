/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Represents a value that is lazily computed and then cached for later usage.

pub const MAX_TRIES: u64 = 0xfff;

/// A wrapper around a value that will be lazily computed and cached.
#[derive(Debug, Clone)]
pub enum RetryCell<T> {
    /// Has not been calculated yet.
    Pending { tries: u64 },
    /// Failed to calculate.
    Unavailable,
    /// Calculated successfully
    Available { value: T },
}

impl<T> Default for RetryCell<T> {
    fn default() -> Self {
        Self::Pending { tries: MAX_TRIES }
    }
}

#[allow(unused)]
impl<T> RetryCell<T> {
    pub fn get_or_try_init<F>(&mut self, init: F) -> anyhow::Result<&T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        match self {
            Self::Pending { tries } => {
                if *tries == 0 {
                    *self = Self::Unavailable;
                    anyhow::bail!("maximum retries hit!");
                }
                *tries -= 1;
                *self = Self::Available { value: init()? };
                match self {
                    Self::Available { value } => Ok(value),
                    _ => unreachable!(),
                }
            }
            Self::Unavailable => {
                anyhow::bail!("maximum retries hit!");
            }
            Self::Available { value } => Ok(value),
        }
    }

    pub fn get_or_try_init_mut<F>(&mut self, init: F) -> anyhow::Result<&mut T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        match self {
            Self::Pending { tries } => {
                if *tries == 0 {
                    *self = Self::Unavailable;
                    anyhow::bail!("maximum retries hit!");
                }
                *tries -= 1;
                *self = Self::Available { value: init()? };
                match self {
                    Self::Available { value } => Ok(value),
                    _ => unreachable!(),
                }
            }
            Self::Unavailable => {
                anyhow::bail!("maximum retries hit!");
            }
            Self::Available { value } => Ok(value),
        }
    }
}
