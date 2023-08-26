//! A multi-threaded code test support library with a focus on exacerbating contention.

#![deny(missing_docs)]

mod bursty;

pub use bursty::{Bursty, BurstyBuilder};
