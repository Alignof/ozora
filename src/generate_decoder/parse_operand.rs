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

/// Group instructions by an imm field value.
fn group_by_imm_value(insns: &Vec<Instruction>, imm_range: &Range<u8>) -> Vec<Vec<Instruction>> {
    let mut insns_map = HashMap::new();
    for insn in insns {
        let key = insn.get_imm_value_by_range(imm_range);
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
    imm_field_list: &[Range<u8>],
    imm_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let imm_field_range = &imm_field_list[imm_index];
    let mut grouped_insns = group_by_imm_value(insns, imm_field_range);

    // skip this level
    if grouped_insns.len() == 1 {
        generate_each_field_pattern(file, ext_name, insns, imm_field_list, imm_index + 1)?;
        return Ok(());
    }

    writeln!(
        file,
        "match op_{end}_{start} {{",
        end = imm_field_range.end,
        start = imm_field_range.start
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

            if let Some(imm_val) = dbg!(insn).get_imm_value_by_range(dbg!(imm_field_range)) {
                indoc::writedoc!(
                    file,
                    "\t\t{imm_val:#0width$b} => {ext_name}Opcode::{},\n",
                    insn_name_upper,
                    width = imm_field_range.len(),
                )?;
            } else {
                indoc::writedoc!(file, "\t\t_ => {ext_name}Opcode::{},\n", insn_name_upper)?;
                is_wild_card_needed = false;
            }
        // non leaf
        } else {
            if let Some(imm_val) = insns[0].get_imm_value_by_range(imm_field_range) {
                write!(
                    file,
                    "\t\t{imm_val:#0width$b} => ",
                    width = imm_field_range.len()
                )?;
            } else {
                write!(file, "\t\t_ => ")?;
                is_wild_card_needed = false;
            }
            generate_each_field_pattern(file, ext_name, &insns, imm_field_list, imm_index + 1)?;
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
    let mut imm_field_set = HashSet::new();
    for insn in insns {
        for immf in insn.get_imm_fields() {
            imm_field_set.insert(immf.range.clone());
        }
    }
    let mut imm_field_list: Vec<Range<u8>> = imm_field_set.into_iter().collect();
    imm_field_list.sort_by(|a, b| {
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
    for imm_field in &imm_field_list {
        writeln!(
            file,
            "let op_{end}_{start}: {typ} = {typ}::try_from(inst.slice({end}, {start})).unwrap(); ",
            typ = if imm_field.len() <= 8 { "u8" } else { "u16" },
            end = imm_field.end,
            start = imm_field.start
        )?;
    }
    generate_each_field_pattern(file, ext_name, insns, &imm_field_list, 0)?;
    indoc::writedoc!(
        file,
        "
        }}

        "
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
    generically_generate_parsing_reg_func(&mut file, "imm", ext_name, insns)?;

    Ok(())
}
