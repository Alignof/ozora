//! Genarate a hypervisor module.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ast_util::csrs::Csr;
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

/// Generate the `EmulateExtension::csr`.
fn generate_csr_handler(
    file: &mut File,
    csrs: &Vec<Csr>,
) -> Result<(), Box<dyn std::error::Error>> {
    for csr in csrs {
        indoc::writedoc!(
            file,
            "
            /// TODO: Write the CSR description
            const CSR_{}: usize = {:#x};
            ",
            csr.name().to_uppercase(),
            csr.number(),
        )?;
    }

    indoc::writedoc!(
        file,
        "

        let hypervisor_data = unsafe {{ HYPERVISOR_DATA.lock() }};
        let mut context = hypervisor_data.get().unwrap().guest().context;

        "
    )?;

    indoc::writedoc!(
        file,
        "
        let csr_num = inst.rs2.unwrap();
        match csr_num {{
        "
    )?;

    for csr in csrs {
        indoc::writedoc!(
            file,
            r#"
            CSR_{csr_upper} => match inst.opc {{
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRW)  => todo!("[{csr_lower}] implement CSRRW instruction emulation"),
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRS)  => todo!("[{csr_lower}] implement CSRRS instruction emulation"),
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRC)  => todo!("[{csr_lower}] implement CSRRC instruction emulation"),
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRWI) => todo!("[{csr_lower}] implement CSRRWI instruction emulation"),
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRSI) => todo!("[{csr_lower}] implement CSRRSI instruction emulation"),
                OpcodeKind::Zicsr(ZicsrOpcode::CSRRCI) => todo!("[{csr_lower}] implement CSRRCI instruction emulation"),
                _ => unreachable!(),
            }},
            "#,
            csr_upper = csr.name().to_uppercase(),
            csr_lower = csr.name()
        )?;
    }
    indoc::writedoc!(
        file,
        r#"
            unsupported_csr_num => {{
                unimplemented!("unsupported CSRs: {{unsupported_csr_num:#x}}")
            }}
        }}
        "#
    )?;

    Ok(())
}

/// Genarate hypervisor module.
pub fn create_hikami_module(
    ext_name: &str,
    output_path: &PathBuf,
    insns: &Vec<Instruction>,
    csrs: &Vec<Csr>,
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

    indoc::writedoc!(
        file,
        r"
            /// Emulate Zicfiss CSRs access.
            fn csr(&mut self, inst: &Instruction) {{
        ",
    )?;
    generate_csr_handler(&mut file, csrs)?;
    indoc::writedoc!(
        file,
        r"
            }}

        ",
    )?;

    indoc::writedoc!(
        file,
        r#"
            /// Emulate CSR field that already exists.
            fn csr_field(&mut self, inst: &Instruction, write_to_csr_value: u64, read_csr_value: &mut u64) {{
                todo!("Implementing {ext_name} CSR field emulation");
            }}
        }}
        "#,
    )?;

    Ok(())
}
