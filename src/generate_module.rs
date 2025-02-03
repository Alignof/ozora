//! Genarate a hypervisor module.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ast_util::instruction::Instruction;

/// Generate the `EmulateExtension::instruction`.
fn generate_inst_handler(
    file: &mut File,
    ext_name: &str,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    indoc::writedoc!(
        file,
        r"
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
        ",
    )?;

    // generate instruction pattern from insns data.
    for insn in insns {
        indoc::writedoc!(
            file,
            "
            \t\t\tOpcodeKind::{ext_name}({ext_name}Opcode::{}) => todo!(),
            ",
            insn.name.strip_prefix("RISCV_").unwrap_or(&insn.name)
        )?;
    }

    indoc::writedoc!(
        file,
        "
                \t_ => unreachable!(),
            \t}}
        \t}}

        ",
    )?;

    Ok(())
}

/// Genarate hypervisor module.
pub fn create_hikami_module(
    ext_name: &str,
    output_path: &PathBuf,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    writeln!(file, "//! Emulation {ext_name}")?;
    indoc::writedoc!(
        file,
        "

        use super::{{pseudo_vs_exception, EmulateExtension, EmulatedCsr}};
        use crate::HYPERVISOR_DATA;

        use core::cell::OnceCell;
        use spin::Mutex;
        use raki::{{Instruction, OpcodeKind, {ext_name}Opcode, ZicsrOpcode}};

        "
    )?;

    // generate a global data
    indoc::writedoc!(
        file,
        "
        /// Singleton for {ext_name}.
        pub static mut {}_DATA: Mutex<OnceCell<{ext_name}>> = Mutex::new(OnceCell::new());

        ",
        ext_name.to_uppercase()
    )?;

    // generate struct definition.
    indoc::writedoc!(
        file,
        "
        /// Singleton for {ext_name} extension
        pub struct {ext_name};

        impl {ext_name} {{
            /// Constructor for `{ext_name}`.
            pub fn new() -> Self {{
                {ext_name}
            }}
        }}

        "
    )?;

    // generate implementation of ExtensionEmulation trait.
    generate_inst_handler(&mut file, ext_name, insns)?;

    // generate implementation of ExtensionEmulation trait.
    indoc::writedoc!(
        file,
        r#"
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
