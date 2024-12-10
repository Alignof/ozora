//! Genarate a hypervisor module.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Genarate hypervisor module.
pub fn create_hikami_module(
    ext_name: String,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    writeln!(file, "Hello World!")?;

    Ok(())
}
