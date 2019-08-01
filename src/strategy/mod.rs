use crate::{
    errors::*,
    fastfile::{FastFileReader, FastFileReaderBuilder},
};

pub trait ReaderStrategy {
    fn get_reader(&self, ffrb: FastFileReaderBuilder) -> Result<FastFileReader>;
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::DefaultMacOsReaderStrategy as DefaultReaderStrategy;
