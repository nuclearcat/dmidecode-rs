use crate::Opt;
use smbioslib::*;
use std::fmt::Write;
use std::io::Error;

mod dmiopt;

pub fn table_load(_opt: &Opt) -> Result<(SMBiosData, String), Error> {
    let mut output = String::new();

    writeln!(&mut output, "Getting SMBIOS data from IOKit.").unwrap();

    let smbios_table = table_load_from_device()?;

    Ok((smbios_table, output))
}

/// Returns the bytes to write for `--dump-bin`.
///
/// This platform's raw form already carries its own header, so it is written
/// through unchanged.
pub fn raw_dump() -> Result<Vec<u8>, Error> {
    raw_smbios_from_device()
}
