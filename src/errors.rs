use failure::{Backtrace, Context, Fail};
use std::fmt;

/// The error kind for errors that get returned in the crate
#[derive(Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Memory operation failure
    #[fail(display = "memory operation failed: {}", _0)]
    MemOpFailed(&'static str),
    /// File operation failure
    #[fail(display = "file operation failed")]
    FileOpFailed,
    /// Libc function failure
    #[fail(display = "libc function failed: {}", _0)]
    LibcFailed(&'static str),
}

impl Clone for ErrorKind {
    fn clone(&self) -> Self {
        use self::ErrorKind::*;
        match *self {
            MemOpFailed(s) => MemOpFailed(s),
            FileOpFailed => FileOpFailed,
            LibcFailed(s) => LibcFailed(s),
        }
    }
}

/// The error type for errors that get returned in the lookup module
#[derive(Debug)]
pub struct Error {
    pub(crate) inner: Context<ErrorKind>,
}

impl Error {
    /// Get the kind of the error
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error {
            inner: Context::new(self.inner.get_context().clone()),
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> { self.inner.cause() }

    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.inner, f) }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner } }
}

/// Result type for this crate
pub type Result<T> = ::std::result::Result<T, Error>;
