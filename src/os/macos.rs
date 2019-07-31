use crate::{errors::*, fastfile::FastFileReader};

pub fn prepare_file_for_reading(ffr: &FastFileReader) -> Result<()> {
    if ffr.size() < 256 * 1024 * 1024 {
        read_advice()?;
    } else {
        read_rdahead()?;
    }

    Ok(())
}

fn read_advice() -> Result<()> {
    Ok(())
}

fn read_rdahead() -> Result<()> {
    Ok(())
}
