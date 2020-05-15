//! # hop-internal-metrics
//!
//! A lightweight metrics implementation, designed to track counters and gauges.
//!
//! # Examples
//!
//! A metrics instance can used anything as a key that implements `Eq` and
//! `Hash`, so we use an enum key to count a couple things a few times:
//!
//! ```rust
//! use hop_internal_metrics::Metrics;
//!
//! #[derive(Clone, Debug, Eq, Hash, PartialEq)]
//! #[repr(u8)]
//! enum Metric {
//!     Foo = 0,
//!     Bar = 1,
//! }
//!
//! let metrics = Metrics::new();
//!
//! {
//!     let writer = metrics.writer();
//!     writer.increment(Metric::Foo);
//!     writer.increment(Metric::Foo);
//!     writer.increment(Metric::Bar);
//!     writer.increment(Metric::Foo);
//! }
//!
//! let reader = metrics.reader();
//!
//! println!("Foo counter: {:?}", reader.counter(&Metric::Foo));
//! ```

#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::sync::{Arc, Weak};
use core::{fmt::Debug, hash::Hash};
use dashmap::DashMap;

/// A snapshot of the metrics, including a copy of all metrics.
///
/// This is produced by [`Metrics::snapshot`].
///
/// [`Metrics::snapshot`]: struct.Metrics.html#method.snapshot
#[derive(Clone, Debug)]
pub struct Snapshot<T: Eq + Hash> {
    counters: DashMap<T, u64>,
    gauges: DashMap<T, u64>,
}

impl<T: Eq + Hash> Snapshot<T> {
    /// Return an immutable reference to the map of counters.
    ///
    /// Note that while the reference is immutable, the map can be mutated
    /// through interior mutability.
    pub fn counters(&self) -> &DashMap<T, u64> {
        &self.counters
    }
    /// Retrieve a counter by its key, if it exists.
    ///
    /// Returns `None` if the key doesn't exist.
    pub fn counter(&self, counter: &T) -> Option<u64> {
        self.counters.get(counter).as_deref().copied()
    }

    /// Return an immutable reference to the map of gauges.
    ///
    /// Note that while the reference is immutable, the map can be mutated
    /// through interior mutability.
    pub fn gauges(&self) -> &DashMap<T, u64> {
        &self.gauges
    }

    /// Retrieve a gauge by its key, if it exists.
    ///
    /// A gauge with a value of 0 doesn't mean that the gauge hasn't been set,
    /// it means that the gauge was set with a value of 0. Returns `None` if the
    /// gauge doesn't exist.
    pub fn gauge(&self, gauge: &T) -> Option<u64> {
        self.gauges.get(gauge).as_deref().copied()
    }
}

/// A set of metrics contaning counters and gauges.
#[derive(Debug)]
pub struct Metrics<T: Eq + Hash>(Arc<Snapshot<T>>);

impl<T: Eq + Hash> Metrics<T> {
    /// Create a new metrics instance.
    ///
    /// Metrics instances aren't global, so two metrics instances aren't
    /// related.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new reader, which can only be used to read metrics.
    ///
    /// There can be multiple readers.
    pub fn reader(&self) -> Reader<T> {
        Reader {
            metrics: Arc::downgrade(&self.0),
        }
    }

    /// Create a new writer, which can only be used to write to metrics.
    ///
    /// There can be multiple writers.
    pub fn writer(&self) -> Writer<T> {
        Writer {
            metrics: Arc::downgrade(&self.0),
        }
    }
}

impl<T: Clone + Eq + Hash> Metrics<T> {
    /// Create a snapshot of all of the metrics.
    ///
    /// This will clone all of the metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hop_internal_metrics::Metrics;
    ///
    /// let metrics = Metrics::new();
    /// let writer = metrics.writer();
    ///
    /// // increment "foo", setting it to 1
    /// writer.increment("foo");
    ///
    /// let snapshot = metrics.snapshot();
    ///
    /// // increment "foo", setting it to 2
    /// writer.increment("foo");
    ///
    /// // "foo" in the metrics is 2, but in the snapshot it is 1
    /// let reader = metrics.reader();
    /// assert_eq!(reader.counter(&"foo"), Some(2));
    /// assert_eq!(snapshot.counter(&"foo"), Some(1));
    /// ```
    pub fn snapshot(&self) -> Snapshot<T> {
        Snapshot {
            counters: self.0.counters.clone(),
            gauges: self.0.gauges.clone(),
        }
    }
}

impl<T: Eq + Hash> Default for Metrics<T> {
    fn default() -> Self {
        Self(Arc::new(Snapshot {
            counters: DashMap::new(),
            gauges: DashMap::new(),
        }))
    }
}

/// A reader which can only read metrics from its attached instance.
///
/// There can be multiple readers.
#[derive(Clone, Debug)]
pub struct Reader<T: Eq + Hash> {
    metrics: Weak<Snapshot<T>>,
}

impl<T: Eq + Hash> Reader<T> {
    /// Retrieve a counter by its key, if it exists.
    ///
    /// Returns `None` if the key doesn't exist or if the metrics instance no
    /// longer exists. Whether the metrics instance still exists can be checked
    /// via [`Reader::valid`].
    ///
    /// [`Reader::valid`]: #method.valid
    pub fn counter(&self, counter: &T) -> Option<u64> {
        self.metrics
            .upgrade()
            .and_then(|metrics| metrics.counters.get(counter).as_deref().copied())
    }

    /// Retrieve a gauge by its key, if it exists.
    ///
    /// A gauge with a value of 0 doesn't mean that the gauge hasn't been set,
    /// it means that the gauge was set with a value of 0.
    ///
    /// Returns `None` if the key doesn't exist or if the metrics instance no
    /// longer exists. Whether the metrics instance still exists can be checked
    /// via [`Reader::valid`].
    ///
    /// [`Reader::valid`]: #method.valid
    pub fn gauge(&self, gauge: &T) -> Option<u64> {
        self.metrics
            .upgrade()
            .and_then(|metrics| metrics.gauges.get(gauge).as_deref().copied())
    }

    /// Determine whether the reader is still valid, meaning that its associated
    /// metrics instance still exists.
    pub fn valid(&self) -> bool {
        self.metrics.strong_count() > 0
    }
}

/// A writer which can only write metrics to its attached instance.
///
/// There can be multiple writers.
#[derive(Clone, Debug)]
pub struct Writer<T: Eq + Hash> {
    metrics: Weak<Snapshot<T>>,
}

impl<T: Eq + Hash> Writer<T> {
    /// Attempt to write a new value to a gauge, which may or may not already
    /// exist.
    ///
    /// The gauge will be overwritten if it exists, or a new one will be made in
    /// its place if it doesn't.
    ///
    /// Returns whether the gauge could be written. This only fails if the
    /// metrics instance no longer exists.
    pub fn gauge(&self, gauge: T, value: u64) -> bool {
        self.metrics
            .upgrade()
            .map(|metrics| {
                *metrics.gauges.entry(gauge).or_default() = value;
            })
            .is_some()
    }

    /// Attempt to increment a counter, which may or may not already exist.
    ///
    /// The counter will be incremented if it exists, or a new one will be made
    /// in its place if it doesn't. Counters are given an initial value of 1.
    ///
    /// Returns whether the counter could be written. This only fails if the
    /// metrics instance no longer exists.
    pub fn increment(&self, counter: T) -> bool {
        self.metrics
            .upgrade()
            .map(|metrics| {
                *metrics.counters.entry(counter).or_default() += 1;
            })
            .is_some()
    }

    /// Determine whether the writer is still valid, meaning that its associated
    /// metrics instance still exists.
    pub fn valid(&self) -> bool {
        self.metrics.strong_count() > 0
    }
}
