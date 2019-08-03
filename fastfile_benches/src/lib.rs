#![feature(read_initializer)]

pub static FILE_SIZES: &[usize] = &[
    1024,
    4 * 1024,
    16 * 1024,
    256 * 1024,
    1024 * 1024,
    10 * 1024 * 1024,
    100 * 1024 * 1024,
];

pub mod benches;
pub mod io;
