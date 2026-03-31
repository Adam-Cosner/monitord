/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Represents a value that is lazily computed and then cached for later usage.

/// A wrapper around a value that will be lazily computed and cached.
#[derive(Debug, Clone, Default)]
pub enum Discovery<T> {
    #[default]
    /// Has not been calculated yet.
    Pending,
    /// Failed to calculate.
    Unavailable,
    /// Calculated successfully
    Available(T),
}

#[allow(unused)]
impl<T> Discovery<T> {
    /// Try once. On failure, mark as permanently unavailable.
    pub fn probe<F>(&mut self, init: F) -> Option<&T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = match init() {
                Ok(value) => Self::Available(value),
                Err(e) => {
                    tracing::warn!("discovery probe failed: {}", e);
                    Self::Unavailable
                }
            };
        }
        self.get()
    }

    /// Try once. On failure, mark as permanently unavailable.
    pub fn probe_mut<F>(&mut self, init: F) -> Option<&mut T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = match init() {
                Ok(value) => Self::Available(value),
                Err(e) => {
                    tracing::warn!("discovery probe failed: {}", e);
                    Self::Unavailable
                }
            };
        }
        self.get_mut()
    }

    /// Try once. On failure, propagate the error and keep as pending.
    pub fn require<F>(&mut self, init: F) -> anyhow::Result<&T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = Self::Available(init()?);
        }
        self.get()
            .ok_or_else(|| anyhow::anyhow!("Discovery unavailable"))
    }

    /// Try once. On failure, propagate the error and keep as pending.
    pub fn require_mut<F>(&mut self, init: F) -> anyhow::Result<&mut T>
    where
        F: FnOnce() -> anyhow::Result<T>,
    {
        if matches!(self, Self::Pending) {
            *self = Self::Available(init()?);
        }
        self.get_mut()
            .ok_or_else(|| anyhow::anyhow!("Discovery unavailable"))
    }

    /// Get an immutable reference to the value, if available.
    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Available(value) => Some(value),
            _ => None,
        }
    }

    /// Get a mutable reference to the value, if available.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Available(value) => Some(value),
            _ => None,
        }
    }
}
