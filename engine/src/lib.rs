#![deny(clippy::all, clippy::cargo)]
#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod command;
pub mod hop;
pub mod metrics;
pub mod pubsub;
pub mod session;
pub mod state;

mod pool;

pub use hop::Hop;
