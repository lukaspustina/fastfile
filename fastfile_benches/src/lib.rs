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
    1 * 1024 * 1024,
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

pub mod benches;
pub mod io;
pub mod utils;
