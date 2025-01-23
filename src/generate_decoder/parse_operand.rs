//! Generate a parsing operand module.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::ops::Range;
use std::path::Path;

use crate::ast_util::instruction::Instruction;

/// Generate a parsing `rd`, `rs1`, `rs2`, `imm` function.
fn generically_generate_parsing_reg_func(
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
    for reg_field_range in &reg_field_set {
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

/// Group instructions by an opc field value.
fn group_by_opc_value(insns: &Vec<Instruction>, opc_range: &Range<u8>) -> Vec<Vec<Instruction>> {
    let mut insns_map = HashMap::new();
    for insn in insns {
        let key = insn.get_opc_value_by_range(opc_range);
        insns_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(insn.clone());
    }

    insns_map.into_values().collect()
}

/// Generate each field pattern by calling recursively.
fn generate_each_field_pattern(
    file: &mut File,
    ext_name: &str,
    insns: &Vec<Instruction>,
    opc_field_list: &[Range<u8>],
    opc_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let opc_field_range = &opc_field_list[opc_index];
    let mut grouped_insns = group_by_opc_value(insns, opc_field_range);

    // skip this level if it is non leaf
    if grouped_insns.len() == 1 && grouped_insns[0].len() != 1 {
        generate_each_field_pattern(file, ext_name, insns, opc_field_list, opc_index + 1)?;
        return Ok(());
    }

    writeln!(
        file,
        "match op_{end}_{start} {{",
        end = opc_field_range.end,
        start = opc_field_range.start
    )?;

    grouped_insns.sort_by_key(std::vec::Vec::len);
    let mut is_wild_card_needed = true;
    for insns in grouped_insns {
        // leaf
        if insns.len() == 1 {
            let insn = &insns[0];
            let insn_name_upper = insn
                .name
                .strip_prefix("RISCV_")
                .unwrap_or(&insn.name)
                .to_uppercase();

            if let Some(opc_val) = dbg!(insn).get_opc_value_by_range(dbg!(opc_field_range)) {
                indoc::writedoc!(
                    file,
                    "\t\t{opc_val:#0width$b} => {ext_name}Opcode::{},\n",
                    insn_name_upper,
                    width = opc_field_range.len(),
                )?;
            } else {
                indoc::writedoc!(file, "\t\t_ => {ext_name}Opcode::{},\n", insn_name_upper)?;
                is_wild_card_needed = false;
            }
        // non leaf
        } else {
            if let Some(opc_val) = insns[0].get_opc_value_by_range(opc_field_range) {
                write!(
                    file,
                    "\t\t{opc_val:#0width$b} => ",
                    width = opc_field_range.len()
                )?;
            } else {
                write!(file, "\t\t_ => ")?;
                is_wild_card_needed = false;
            }
            generate_each_field_pattern(file, ext_name, &insns, opc_field_list, opc_index + 1)?;
        }
    }

    if is_wild_card_needed {
        writeln!(file, "_ => Err(DecodingError::InvalidOpcode),")?;
    }
    writeln!(file, "}}")?;

    Ok(())
}

/// Generate operand parsing function
fn generate_parsing_opecode_func(
    file: &mut File,
    ext_name: &str,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut opc_field_set = HashSet::new();
    for insn in insns {
        for opcf in insn.get_opc_fields() {
            opc_field_set.insert(opcf.range.clone());
        }
    }
    let mut opc_field_list: Vec<Range<u8>> = opc_field_set.into_iter().collect();
    opc_field_list.sort_by(|a, b| {
        if a.start == b.start {
            b.end.cmp(&a.end)
        } else {
            a.start.cmp(&b.start)
        }
    });
    indoc::writedoc!(
        file,
        "
        pub fn parse_opcode(inst: u32) -> Result<{ext_name}Opcode, DecodingError> {{
        "
    )?;
    for opc_field in &opc_field_list {
        writeln!(
            file,
            "let op_{end}_{start}: {typ} = {typ}::try_from(inst.slice({end}, {start})).unwrap(); ",
            typ = if opc_field.len() <= 8 { "u8" } else { "u16" },
            end = opc_field.end,
            start = opc_field.start
        )?;
    }
    generate_each_field_pattern(file, ext_name, insns, &opc_field_list, 0)?;
    indoc::writedoc!(
        file,
        "
        }}

        "
    )?;

    Ok(())
}

/// Generate unit tests for decoder.
fn generate_unit_tests(
    file: &mut File,
    ext_name: &str,
    insns: &Vec<Instruction>,
) -> Result<(), Box<dyn std::error::Error>> {
    indoc::writedoc!(
        file,
        "
        #[cfg(test)]
        #[allow(unused_variables)]
        mod test_{ext_name} {{
            #[test]
            #[allow(overflowing_literals)]
            fn {ext_name}_32bit_decode_test() {{
                use super::*;
                use crate::{{Decode, Isa, OpcodeKind}};

                let test_32 = |inst_32: u32,
                               expected_op: OpcodeKind,
                               expected_rd: Option<usize>,
                               expected_rs1: Option<usize>,
                               expected_rs2: Option<usize>,
                               expected_imm: Option<i32>| {{
                    let op_32 = inst_32.parse_opcode(Isa::Rv64).unwrap();
                    assert_eq!(op_32, expected_op);
                    assert_eq!(inst_32.parse_rd(&op_32).unwrap(), expected_rd);
                    assert_eq!(inst_32.parse_rs1(&op_32).unwrap(), expected_rs1);
                    assert_eq!(inst_32.parse_rs2(&op_32).unwrap(), expected_rs2);
                    assert_eq!(inst_32.parse_imm(&op_32, Isa::Rv64).unwrap(), expected_imm);
                }};
        ",
        ext_name = ext_name.to_lowercase(),
    )?;

    for insn in insns {
        let (insn_val, rd, rs1, rs2, imm) = insn.get_random_insn_value();
        indoc::writedoc!(
            file,
            "
                test_32(
                    {insn_val:#032b},
                    OpcodeKind::{ext_name}({ext_name}Opcode::{}),
                    {rd:?},
                    {rs1:?},
                    {rs2:?},
                    {imm:?},
                );
            ",
            insn.name
                .strip_prefix("RISCV_")
                .unwrap_or(&insn.name)
                .to_uppercase(),
        )?;
    }
    indoc::writedoc!(
        file,
        "
            }}
        }}
        ",
    )?;

    Ok(())
}

/// Generate operand parser.
pub fn create_raki_decoder(
    ext_name: &str,
    output_path: &Path,
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

    generate_parsing_opecode_func(&mut file, ext_name, insns)?;

    generically_generate_parsing_reg_func(&mut file, "rd", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "rs1", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "rs2", ext_name, insns)?;
    generically_generate_parsing_reg_func(&mut file, "opc", ext_name, insns)?;

    generate_unit_tests(&mut file, ext_name, insns)?;

    Ok(())
}
