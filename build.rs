use libc;
use std::{
    env,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

fn main() {
    let output_dir = env::var("OUT_DIR").expect("Could not determine OUT_DIR from environment");

    generate_os_consts_file(output_dir).expect("Failed to write OS consts file");
}

fn generate_os_consts_file<T: AsRef<Path>>(output_path: T) -> io::Result<()> {
    let path = output_path.as_ref().join("os_consts.rs");
    let mut file = BufWriter::new(File::create(&path)?);

    let page_size: usize = get_system_page_size() as usize;
    writeln!(&mut file, "pub const PAGE_SIZE: usize = {};", page_size)?;

    Ok(())
}

fn get_system_page_size() -> ::libc::c_long {
    unsafe { ::libc::sysconf(libc::_SC_PAGESIZE) }
}
