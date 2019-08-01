//! # Fastfile
//! `fastfile` is a crate for reading and writing as fast as possible.
//!
//! `fastfile` uses a heuristic to choose the fastest strategy taking several parameters into
//! account. For example, the file size, the file system type, and the operating system.

#[deny(missing_docs)]

/// Errors
pub mod errors;

/// The `fastfile` module contains the FastFile type
pub mod fastfile;

/// Internal abstraction of OS specific function
mod os;

/// OS specific file IO strategies
mod strategy;

/// `prelude` for the most important types and functions
pub mod prelude {
    pub use crate::{
        fastfile::{FastFile, MAX_READ_BUF_SIZE},
        prepare_buf,
    };
}
