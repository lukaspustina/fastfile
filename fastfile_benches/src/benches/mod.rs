use crate::{benchmark::Param, utils::create_random_test_file};

pub mod methods;

use byte_unit::Byte;
use std::{fs, io, path::PathBuf};

#[rustfmt::skip]
pub static FILE_SIZES_VERY_SMALL: &[usize] = &[
    1024,
    2 * 1024,
    4 * 1024,
    8 * 1024,
    16 * 1024,
    64 * 1024,
    128 * 1024,
];

#[rustfmt::skip]
pub static FILE_SIZES_SMALL: &[usize] = &[
    1024,
    2 * 1024,
    4 * 1024,
    8 * 1024,
    16 * 1024,
    64 * 1024,
    128 * 1024,
    256 * 1024,
    512 * 1024,
    1024 * 1024,
    2 * 1024 * 1024,
];

#[rustfmt::skip]
pub static FILE_SIZES_MEDIUM: &[usize] = &[
    2 * 1024 * 1024,
    8 * 1024 * 1024,
    10 * 1024 * 1024,
    16 * 1024 * 1024,
    25 * 1024 * 1024,
    32 * 1024 * 1024,
    50 * 1024 * 1024,
    64 * 1024 * 1024,
    100 * 1024 * 1024,
    128 * 1024 * 1024,
    256 * 1024 * 1024,
];

#[rustfmt::skip]
pub static FILE_SIZES_LARGE: &[usize] = &[
    256 * 1024 * 1024,
    512 * 1024 * 1024,
    1024 * 1024 * 1024,
];

pub fn prepare(file_sizes: &[usize]) -> io::Result<Vec<Param<PathBuf>>> {
    let mut params = Vec::with_capacity(file_sizes.len());

    for &size in file_sizes {
        let name = format!("{}", size);
        let bytes = Byte::from_bytes(size as u128);
        let display_name = bytes.get_appropriate_unit(true).format(0).to_string();
        let path = create_random_test_file(size)?;
        let p = Param::new(name, display_name, size, path);
        params.push(p);
    }

    Ok(params)
}

pub fn cleanup(params: Vec<Param<PathBuf>>) -> io::Result<()> {
    for p in params {
        fs::remove_file(p.value())?;
    }

    Ok(())
}
