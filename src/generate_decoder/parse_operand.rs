//! Generate a parsing operand module.

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ast_util::instruction::Instruction;

/// Generate a parsing `rd`, `rs1`, `rs2`, `imm` function.
pub fn generically_generate_parsing_reg_func(
    file: &mut File,
    reg_type: &str,
    ext_name: &str,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    indoc::writedoc!(
        file,
        "
        /// Parsing {ext_name} instruction's {reg_type}
        pub fn parse_{reg_type}(inst: u32, opkind: &{ext_name}Opcode) -> Option<usize> {{
        "
    )?;

    let mut reg_field_set = HashSet::new();
    for insn in insns {
        if let Some(field) = insn.get_field_by_name(reg_type) {
            reg_field_set.insert(field.range.clone());
        }
    }
    for reg_field_range in reg_field_set.iter() {
        indoc::writedoc!(
            file,
            "
            \tlet {reg_type}_{end}_{start}: usize = inst.slice({end}, {start}) as usize;
            ",
            start = reg_field_range.start,
            end = reg_field_range.end,
        )?;
    }
    writeln!(file, "\tmatch opkind {{")?;
    for insn in insns {
        indoc::writedoc!(
            file,
            "\t\t{ext_name}Opcode::{} => {}\n",
            insn.name
                .strip_prefix("RISCV_")
                .unwrap_or(&insn.name)
                .to_uppercase(),
            match insn.get_field_by_name(reg_type) {
                Some(reg_field) => format!(
                    "Some({reg_type}_{end}_{start}),",
                    start = reg_field.range.start,
                    end = reg_field.range.end,
                ),
                None => "None,".to_string(),
            }
        )?;
    }
    indoc::writedoc!(
        file,
        "
            }}
        }}

        "
    )?;

    Ok(())
}

/// Genarate operand parser.
pub fn create_raki_decoder(
    ext_name: &str,
    output_path: &PathBuf,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir_name = "decode";
    let _ = std::fs::create_dir(output_path.with_file_name(dir_name));
    let mut file = File::create(output_path.with_file_name(format!("{dir_name}/{ext_name}.rs")))?;

    // Document of a module and import statements.
    indoc::writedoc!(
        file,
        "
        //! {ext_name} extension decoder

        use super::super::{{DecodeUtil, DecodingError}};
        use crate::instruction::zicfiss_extension::ZicfissOpcode;

        "
    )?;

    // `parse_opcode`
    indoc::writedoc!(
        file,
        "
        pub fn parse_opcode(inst: u32) -> Result<ZicfissOpcode, DecodingError> {{
            let opmap: u8 = u8::try_from(inst.slice(6, 0)).unwrap();
        }}

        "
    )?;

    generically_generate_parsing_reg_func(&mut file, "rd", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "rs1", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "rs2", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "imm", ext_name, insns)?;

    Ok(())
}
