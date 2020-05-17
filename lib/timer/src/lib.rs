//! # hop-internal-timer
//!
//! A high resolution timer which calls in to `libc::clock_gettime` using the
//! monotonic clock.
//!
//! The timer can be started and stopped, and when stopped provides the duration
//! from start to finish.
//!
//! This timer is `no_std` and no-`alloc`, utilizing only
//! `core::convert::TryFrom`, `core::time::Duration`, and `libc`.
//!
//! # Examples
//!
//! ```
//! use hop_internal_timer::Timer;
//!
//! # fn try_main() -> Option<()> {
//! let mut timer = Timer::new();
//!
//! // you can check if the timer is running, which it's not
//! assert!(!timer.is_running());
//!
//! // let's start the timer. starting it gives you back whether it successfully
//! // started, which would fail if the timer was already started
//! assert!(timer.start());
//!
//! // you can check that the timer is in fact running now
//! assert!(timer.is_running());
//!
//! // when you stop the timer you get back a duration, so let's print it
//! println!("Time to run: {:?}", timer.stop()?);
//!
//! // it's not running anymore
//! assert!(!timer.is_running());
//!
//! // and you can get the duration of the timer again if you need to
//! assert!(timer.duration().is_some());
//!
//! // if you want to re-use the timer, you can, so you don't need to
//! // re-instantiate it
//! timer.reset();
//!
//! // now you can start the cycle all over again, and the duration is no longer
//! // available
//! assert!(timer.duration().is_none());
//! # Some(()) }
//! # fn main() { try_main().unwrap(); }
//! ```

#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions, clippy::needless_doctest_main)]
#![no_std]

use core::{convert::TryFrom, time::Duration};
use libc::{timespec, CLOCK_MONOTONIC};

/// A timer backed by libc's monotonic clock and `clock_gettime(3)`.
///
/// The timer supports being reset, calculating the duration afterwards,
/// resetting itself to a clean slate, and determining if it's still running
/// (counting time, meaning that it's unfinished).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Timer {
    start: Option<Duration>,
    stop: Option<Duration>,
}

impl Timer {
    /// Creates a new timer, which can be started and then later stopped,
    /// returning the duration of the running time.
    ///
    /// # Examples
    ///
    /// Start a timer, do some stuff, and then stop it, printing the time it
    /// took to do the stuff:
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// # fn try_main() -> Option<()> {
    /// let mut timer = Timer::new();
    /// timer.start();
    ///
    /// // do some stuff
    ///
    /// let duration = timer.stop()?;
    /// println!("Time to do the stuff: {:?}", duration);
    /// # Some(()) }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the timer.
    ///
    /// Returns whether the timer was started. This only returns `false` if the
    /// timer was already started.
    ///
    /// # Examples
    ///
    /// Start a timer twice in a row, asserting the first time that it
    /// successfully started and then asserting the second time that it didn't
    /// successfully start due to already being started:
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// let mut timer = Timer::new();
    /// assert!(timer.start());
    /// assert!(!timer.start());
    /// ```
    pub fn start(&mut self) -> bool {
        if self.start.is_some() {
            false
        } else {
            self.start.replace(now());

            true
        }
    }

    /// Stops the timer, returning the time that it took to drive to completion.
    ///
    /// Returns `None` if the timer wasn't started or already finished.
    ///
    /// # Examples
    ///
    /// Start a timer and then stop it twice in a row, asserting the first time
    /// that it successfully stopped and then asserting the second time that it
    /// didn't successfully stop due to already being stopped:
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// let mut timer = Timer::new();
    /// timer.start();
    ///
    /// // assert that stopping the timer gives us the duration
    /// assert!(timer.stop().is_some());
    /// // and that stopping it again gives us nothing, since it already
    /// // finished
    /// assert!(timer.stop().is_none());
    /// ```
    pub fn stop(&mut self) -> Option<Duration> {
        if self.start.is_none() || self.stop.is_some() {
            return None;
        }

        self.stop.replace(now());

        self.duration()
    }

    /// Calculates the duration of the timer if it has finished.
    ///
    /// The returned calculation is the difference between the stop and the start
    /// times. If the timer was never started or never stopped, then `None` is
    /// returned.
    ///
    /// # Examples
    ///
    /// Check the duration after stopping the timer:
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// # fn try_main() -> Option<()> {
    /// let mut timer = Timer::new();
    /// timer.start();
    /// let duration = timer.stop()?;
    /// assert_eq!(timer.duration()?, duration);
    /// # Some(()) }
    /// # fn main() { try_main().unwrap(); }
    /// ```
    pub fn duration(&self) -> Option<Duration> {
        Some(self.stop? - self.start?)
    }

    /// Determines whether the timer is currently running.
    ///
    /// # Examples
    ///
    /// Start a timer and then make sure it's running, stop it, and then confirm
    /// that it's no longer running:
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// let mut timer = Timer::new();
    /// timer.start();
    /// assert!(timer.is_running());
    ///
    /// timer.stop();
    ///
    /// // the timer is no longer running
    /// assert!(!timer.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        self.start.is_some() && self.stop.is_none()
    }

    /// Resets the timer.
    ///
    /// This *completely* resets it, acting as a new timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use hop_internal_timer::Timer;
    ///
    /// let mut timer = Timer::new();
    /// // check that the timer started
    /// assert!(timer.start());
    ///
    /// // oops, something happened, resetting it will stop it
    /// timer.reset();
    /// assert!(!timer.is_running());
    /// ```
    pub fn reset(&mut self) {
        self.stop.take();
        self.start.take();
    }
}

#[cfg(feature = "__internal_test")]
#[inline(always)]
pub fn test_now() -> Duration {
    now()
}

// We'll just slightly overload the usage of core's duration here. :) The
// argument validating this can be that it's the duration since boot!
fn now() -> Duration {
    let timespec = monotonic_time();

    parse_timespec(timespec)
}

fn parse_timespec(timespec: timespec) -> Duration {
    let mut s = u64::try_from(timespec.tv_sec).unwrap_or(0);
    let ns = u32::try_from(timespec.tv_nsec).unwrap_or_else(|_| {
        if timespec.tv_nsec < 0 {
            u32::MIN
        } else {
            s = s.saturating_sub(1);

            999_999_999
        }
    });

    Duration::new(s, ns)
}

fn monotonic_time() -> timespec {
    let mut timespec = timespec {
        tv_nsec: 0,
        tv_sec: 0,
    };

    unsafe {
        libc::clock_gettime(CLOCK_MONOTONIC, &mut timespec);
    }

    timespec
}

#[cfg(test)]
mod tests {
    use super::Timer;
    use core::time::Duration;
    use libc::timespec;

    #[test]
    fn test_now() {
        let time = super::now();
        assert!(time.as_secs() > 0);
    }

    #[cfg(feature = "__internal_test")]
    #[test]
    fn test_exported_now() {
        let time = super::now();
        assert!(time.as_secs() > 0);
    }

    #[test]
    fn test_parse_timespec_corrects_negative_seconds() {
        let time = timespec {
            tv_sec: -1,
            tv_nsec: 5_000_000,
        };

        assert_eq!(super::parse_timespec(time), Duration::new(0, 5_000_000));
    }

    #[test]
    fn test_parse_timespec_corrects_overflowing_nanos() {
        let time = timespec {
            tv_nsec: i64::MAX,
            tv_sec: 3,
        };

        assert_eq!(super::parse_timespec(time), Duration::new(2, 999_999_999));
    }

    #[test]
    fn test_parse_timespec_corrects_negative_nanos() {
        let time = timespec {
            tv_nsec: -1,
            tv_sec: 3,
        };

        assert_eq!(super::parse_timespec(time), Duration::new(3, 0));
    }

    #[test]
    fn test_new_defaults() {
        let timer = Timer::new();
        assert_eq!(timer, Timer::default());
        assert!(timer.start.is_none());
        assert!(timer.stop.is_none());
    }

    #[test]
    fn test_start_cant_start_again() {
        let mut timer = Timer::new();
        assert!(timer.start.is_none());
        assert!(timer.start());

        let start = timer.start.expect("start is set");
        assert!(!timer.start());
        assert_eq!(timer.start.expect("start is set"), start);
    }

    #[test]
    fn test_stop_doesnt_stop_again() {
        let mut timer = Timer::new();
        timer.start();
        assert!(timer.stop().is_some());
        assert!(timer.stop().is_none());
    }

    #[test]
    fn test_reset_is_equivalent_to_new() {
        let mut timer = Timer::new();
        timer.start();
        timer.stop();
        timer.reset();
        assert_eq!(timer, Timer::new());
    }

    #[test]
    fn test_reset_resets_running_timer() {
        let mut timer = Timer::new();
        timer.start();
        timer.reset();
        assert_eq!(timer, Timer::new());
    }

    #[test]
    fn test_duration_requires_finish() {
        let mut timer = Timer::new();
        assert!(timer.duration().is_none());
        timer.start();
        assert!(timer.duration().is_none());
        timer.stop();
        assert!(timer.duration().is_some());
    }

    #[test]
    fn test_is_running() {
        let mut timer = Timer::new();
        assert!(!timer.is_running());
        timer.start();
        assert!(timer.is_running());
        timer.stop();
        assert!(!timer.is_running());
    }
}
