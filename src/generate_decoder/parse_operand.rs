//! Generate a parsing operand module.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::ast_util::instruction::Instruction;

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

    // `parse_rd`
    indoc::writedoc!(
        file,
        "
        pub fn parse_rd(inst: u32, opkind: &ZicfissOpcode) -> Option<usize> {{
        "
    )?;
    indoc::writedoc!(
        file,
        "
        let rd: usize = inst.slice(11, 7) as usize;
        "
    )?;
    indoc::writedoc!(
        file,
        "
        }}

        "
    )?;

    // `parse_rs1`
    indoc::writedoc!(
        file,
        "
        pub fn parse_rs1(inst: u32, opkind: &ZicfissOpcode) -> Option<usize> {{
            let rs1: usize = inst.slice(19, 15) as usize;
        }}

        "
    )?;

    // `parse_rs2`
    indoc::writedoc!(
        file,
        "
        pub fn parse_rs2(inst: u32, opkind: &ZicfissOpcode) -> Option<usize> {{
            let rs2: usize = inst.slice(24, 20) as usize;
        }}

        "
    )?;

    // `parse_imm`
    indoc::writedoc!(
        file,
        "
        pub fn parse_imm(inst: u32, opkind: &ZicfissOpcode) -> Option<i32> {{
        }}

        "
    )?;

    Ok(())
}
