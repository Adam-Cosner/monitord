/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the implementation of a "lazy-init try once" cache for values that are expensive to compute.

#[derive(Debug, Clone, Default)]
pub enum Cached<T> {
    #[default]
    Pending, // Has not been calculated yet
    Unavailable,  // Failed to calculate
    Available(T), // Calculated successfully
}

impl<T> Cached<T> {
    /// Try once, on failure, log, mark Unavailable, return None
    pub fn get_or_try<F>(&mut self, init: F) -> Option<&T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = match init() {
                Ok(value) => Self::Available(value),
                Err(e) => {
                    tracing::warn!("Discovery failed: {e}");
                    Self::Unavailable
                }
            };
        }
        match self {
            Self::Available(value) => Some(value),
            _ => None,
        }
    }

    /// Same as `get_or_try` but returns a &mut T
    pub fn get_or_try_mut<F>(&mut self, init: F) -> Option<&mut T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = match init() {
                Ok(value) => Self::Available(value),
                Err(e) => {
                    tracing::warn!("Discovery failed: {e}");
                    Self::Unavailable
                }
            };
        }
        match self {
            Self::Available(value) => Some(value),
            _ => None,
        }
    }

    /// Try once, on failure, propagate the error (stays Pending for retry)
    pub fn get_or_require<F>(&mut self, init: F) -> anyhow::Result<&T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = Self::Available(init()?);
        }
        match self {
            Self::Available(value) => Ok(value),
            _ => unreachable!(),
        }
    }
}
