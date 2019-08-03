#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::read_advise;
#[cfg(target_os = "macos")]
pub use macos::read_ahead;
