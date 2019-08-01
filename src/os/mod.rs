

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub(crate) use macos::prepare_file_for_reading;
#[cfg(target_os = "macos")]
pub(crate) use macos::create_backing_reader;
#[cfg(target_os = "macos")]
pub use macos::MacOsBackendStrategySelector as DefaultBackendStrategySelector;

