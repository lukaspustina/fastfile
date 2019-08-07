#![feature(read_initializer)]

pub static FILE_SIZES: &[usize] = &[
    1024,
    2 * 1024,
    4 * 1024,
    8 * 1024,
    16 * 1024,
    64 * 1024,
    256 * 1024,
    1 * 1024 * 1024,
    2 * 1024 * 1024,
    8 * 1024 * 1024,
    10 * 1024 * 1024,
    50 * 1024 * 1024,
    100 * 1024 * 1024,
    200 * 1024 * 1024,
];

pub mod benches;
pub mod io;
pub mod utils;
