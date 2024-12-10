//! Genarate a hypervisor module.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Importing module statements.
const IMPORT_MODULE: &str = r#"
use super::{{pseudo_vs_exception, EmulateExtension, EmulatedCsr}};
use crate::HYPERVISOR_DATA;

use core::cell::OnceCell;
use spin::Mutex;
//use raki::{{Instruction, OpcodeKind, ZicfissOpcode, ZicsrOpcode}};
"#;

/// Genarate hypervisor module.
pub fn create_hikami_module(
    ext_name: String,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    writeln!(file, "//! Emulation {ext_name}")?;
    writeln!(file, "{IMPORT_MODULE}")?;

    // generate a global data
    writeln!(
        file,
        r#"
/// Singleton for {ext_name}.
pub static mut {}_DATA: Mutex<OnceCell<{ext_name}>> = Mutex::new(OnceCell::new());
"#,
        ext_name.to_uppercase()
    )?;

    // generate struct definition.
    writeln!(
        file,
        r#"
/// Singleton for {ext_name} extension
pub struct {ext_name};

impl {ext_name} {{
    /// Constructor for `{ext_name}`.
    pub fn new() -> Self {{
        {ext_name}
    }}
}}
"#,
    )?;

    // generate implementation of ExtensionEmulation trait.
    writeln!(
        file,
        r#"
impl EmulateExtension for {ext_name} {{
    /// Emulate {ext_name} instruction.
    #[allow(clippy::cast_possible_truncation)]
    fn instruction(&mut self, inst: &Instruction) {{
        let mut context = unsafe {{ HYPERVISOR_DATA.lock() }}
            .get()
            .unwrap()
            .guest()
            .context;

        match inst.opc {{
            _ => todo!("Implementing {ext_name} instruction emulation"),
        }}
    }}

    /// Emulate Zicfiss CSRs access.
    fn csr(&mut self, inst: &Instruction) {{
        todo!("Implementing {ext_name} CSR emulation");
    }}

    /// Emulate CSR field that already exists.
    fn csr_field(&mut self, inst: &Instruction, write_to_csr_value: u64, read_csr_value: &mut u64) {{
        todo!("Implementing {ext_name} CSR field emulation");
    }}
}}
"#,
    )?;
    Ok(())
}
