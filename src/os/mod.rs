#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::read_advice;
#[cfg(target_os = "macos")]
pub use macos::read_rdahead;
