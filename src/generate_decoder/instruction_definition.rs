//! Generate a instruction definition.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ast_util::instruction::Instruction;

/// Genarate decoder implementation.
pub fn create_raki_insn_def(
    ext_name: &str,
    output_path: &PathBuf,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir_name = "instruction";
    let _ = std::fs::create_dir(output_path.with_file_name(dir_name));
    let mut file = File::create(output_path.with_file_name(format!("{dir_name}/{ext_name}.rs")))?;

    // Doc comment and import statements.
    indoc::writedoc!(
        file,
        "
        //! {ext_name} extension Instruction

        use super::{{InstFormat, Opcode}};
        use core::fmt::{{self, Display, Formatter}};

        "
    )?;

    // Insturction definitions.
    indoc::writedoc!(
        file,
        "
        /// Insturctions in {ext_name} Extension.
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Debug, PartialEq)]
        pub enum {ext_name}Opcode {{
        "
    )?;
    for insn in insns {
        indoc::writedoc!(
            file,
            "
            \t/// TODO: Add a description of the instruction here.
            \t{},

            ",
            insn.name
                .strip_prefix("RISCV_")
                .unwrap_or(&insn.name)
                .to_uppercase(),
        )?;
    }
    writeln!(file, "}}")?;

    // `Display` tarit implementation.
    indoc::writedoc!(
        file,
        "

        impl Display for ZicfissOpcode {{
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {{
                match self {{
        "
    )?;
    for insn in insns {
        let stripped_insn = insn.name.strip_prefix("RISCV_").unwrap_or(&insn.name);
        indoc::writedoc!(
            file,
            "
            \t\t\t{ext_name}Opcode::{} => write!(f, \"{}\"),
            ",
            stripped_insn.to_uppercase(),
            stripped_insn.to_lowercase(),
        )?;
    }
    indoc::writedoc!(
        file,
        "
                }}
            }}
        }}
        "
    )?;

    // `Opcode` tarit implementation.
    indoc::writedoc!(
        file,
        "

        impl Opcode for ZicfissOpcode {{
            fn get_format(&self) -> InstFormat {{
                match self {{
        "
    )?;
    for insn in insns {
        indoc::writedoc!(
            file,
            "
            \t\t\t{ext_name}Opcode::{} => InstFormat:: ,
            ",
            insn.name
                .strip_prefix("RISCV_")
                .unwrap_or(&insn.name)
                .to_uppercase()
        )?;
    }
    indoc::writedoc!(
        file,
        "
                }}
            }}
        }}
        "
    )?;

    Ok(())
}
