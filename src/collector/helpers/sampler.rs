/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Representation of data that is sampled at regular intervals and diffed after each new sample is taken.
//! When data is `push`ed into the sampler, it mutates the stored value and returns a delta if there was a previous sample.
use std::time::{Duration, Instant};

/// Regularly sampled data helper type.
/// The type must implement the `Differential` trait.
#[derive(Debug, Clone)]
pub struct Sampler<T: Differential> {
    last: Option<Sample<T>>,
}

impl<T: Differential> Sampler<T> {
    /// Initializes a new sampler with no previous sample.
    pub fn new() -> Self {
        Self { last: None }
    }

    /// Replaces the current sample with the given value and returns a delta if there was a previous sample.
    pub fn push(&mut self, value: T) -> Option<Delta<T::Delta>> {
        let now = Instant::now();
        let delta = self.last.take().map(|last| Delta {
            change: value.delta(&last.value),
            interval: now - last.taken_at,
        });
        self.last = Some(Sample {
            value,
            taken_at: now,
        });
        delta
    }
}

/// The total change over the period between two samples.
pub struct Delta<D> {
    /// The change between two samples
    pub change: D,
    /// The interval between the two samples
    pub interval: Duration,
}

/// Internal wrapper around a sample to store the value and the time it was taken.
#[derive(Debug, Clone)]
struct Sample<T> {
    value: T,
    taken_at: Instant,
}

/// Trait for types that can be sampled and diffed.
pub trait Differential {
    /// The type that is used to represent the delta between two samples.
    type Delta;
    /// Calculates the delta between two samples.
    fn delta(&self, previous: &Self) -> Self::Delta;
}
