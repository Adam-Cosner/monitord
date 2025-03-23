use crate::error::Result;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// The `Collector` trait defines the interface for all data collectors.
/// Implementors of this trait should provide methods to collect specific system metrics.
pub trait Collector {
    /// The type of data this collector produces
    type Data;

    /// The configuration type for this collector
    type Config;

    /// Creates a new instance of the collector with the given configuration
    fn new(config: Self::Config) -> Result<Self>
    where
        Self: Sized;

    /// Collects a single data point
    fn collect(&mut self) -> Result<Self::Data>;

    /// Creates a stream that produces data at the specified interval
    fn stream(self, interval: Duration) -> CollectorStream<Self>
    where
        Self: Sized,
    {
        CollectorStream::new(self, interval)
    }
}

/// A stream adapter for collectors that emits data at a specified interval
pub struct CollectorStream<C> {
    collector: C,
    interval: Duration,
    next_poll: std::time::Instant,
}

impl<C, D> CollectorStream<C>
where
    C: Collector<Data = D>,
{
    /// Creates a new collector stream with the specified collector and interval
    pub fn new(collector: C, interval: Duration) -> Self {
        Self {
            collector,
            interval,
            next_poll: std::time::Instant::now(),
        }
    }
}

impl<C, D> Stream for CollectorStream<C>
where
    C: Collector<Data = D> + Unpin,
{
    type Item = Result<D>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let now = std::time::Instant::now();
        if now < this.next_poll {
            // Not time to poll yet, schedule a wakeup
            cx.waker().wake_by_ref();
            futures::task::noop_waker_ref();
            return Poll::Pending;
        }

        // Update next poll time
        this.next_poll = now + this.interval;

        // Collect data
        match this.collector.collect() {
            Ok(data) => Poll::Ready(Some(Ok(data))),
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
