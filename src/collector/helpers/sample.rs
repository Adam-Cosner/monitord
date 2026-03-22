/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A simple struct representing timestamped samples.

#[derive(Debug, Clone)]
pub struct Sample<T> {
    pub value: T,
    pub timestamp: std::time::Instant,
}

pub struct Diff<D> {
    pub delta: D,
    pub elapsed: std::time::Duration,
}

impl<T> Sample<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: std::time::Instant::now(),
        }
    }
}

pub trait Diffable {
    type Delta;
    fn diff(&self, other: &Self) -> Self::Delta;
}

impl<T: Diffable> core::ops::Sub<&Sample<T>> for &Sample<T> {
    type Output = Diff<T::Delta>;

    fn sub(self, other: &Sample<T>) -> Diff<T::Delta> {
        Diff {
            delta: self.value.diff(&other.value),
            elapsed: self.timestamp.elapsed(),
        }
    }
}
