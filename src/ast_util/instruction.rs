use std::ops::Range;
use std::path::Path;

use sailrs::sail_ast::{
    DefinitionAux, Expression, ExpressionAux, Identifier, LiteralAux, Location,
    NumericExpressionAux, Pattern, PatternAux, PatternMatchAux, TypArgAux, TypAux,
    TypeDefinitionAux, TypeUnion,
};
use sailrs::types::ListVec;

use super::unwrap_ident;
use crate::AST;

/// Union clause definition for AST.
///
/// e.g. `RISCV_SLLIUW`, `ZBA_RTYPE`
#[derive(Debug)]
pub struct InstType {
    /// Union name
    name: String,
    /// Referenced instructions.
    _insts: Option<ListVec<Identifier>>,
    /// Index of the instruction.
    index: Option<usize>,
}

/// Operand data
pub struct NamedOperand {
    /// Field name
    name: String,
    /// Bit map of the field.
    range: Range<u8>,
}

/// Immediate data
pub struct Immediate {
    /// Immediate value
    value: u32,
    /// Bit map of the field.
    range: Range<u8>,
}

pub enum Operand {
    Imm(Immediate),
    Named(NamedOperand),
}

/// Instruction data
pub struct Instruction {
    /// Instruction name.
    name: String,

    /// Group name (Name of `InstType`)
    group_name: Option<String>,

    /// List of operands.
    operands: Vec<Operand>,
}

impl InstType {
    pub fn new(type_union: TypeUnion) -> Self {
        let name = unwrap_ident(&type_union.inner.id).to_string();

        if let TypAux::Tuple(list_typ) = *type_union.inner.typ.inner {
            for (index, arg_typ) in list_typ.iter().enumerate() {
                if let TypAux::Id(ident) = *arg_typ.inner.clone() {
                    if let Some(insts) = AST.get().unwrap().get_enum_variants(&ident.inner) {
                        return InstType {
                            name,
                            _insts: Some(insts),
                            index: Some(index),
                        };
                    }
                }
            }
        }

        InstType {
            name,
            _insts: None,
            index: None,
        }
    }
}

/// Get ast node that is contained in target file.
#[allow(dead_code)]
pub fn get_insns_in_target_file(target_file_name: &str) -> Vec<InstType> {
    let ast_node = AST.get().unwrap().get_ast_node().unwrap();
    let mut type_vec = Vec::new();
    if let DefinitionAux::Type(ref type_def) = ast_node.definition {
        if let TypeDefinitionAux::Variant(_ident, _typ_quant, union_list, _flag) = &type_def.inner {
            for type_union in union_list.iter() {
                if let Location::Range(pos, _) = &type_union.annotation.loc {
                    let contain_file = Path::new(pos.pos_fname.as_ref()).file_name().unwrap();
                    if contain_file == target_file_name {
                        type_vec.push(InstType::new(type_union.clone()));
                    }
                }
            }
        }
    }

    type_vec
}

/// Show lhs of the encode data.
pub fn show_encoding_rule_lhs(ident: &Identifier, inst: &InstType, pat_list: &ListVec<Pattern>) {
    match inst.index {
        Some(index) => {
            if let PatternAux::Tuple(union_args) = *pat_list.iter().next().unwrap().inner.clone() {
                if let PatternAux::Identifier(union_ident) =
                    *union_args.iter().nth(index).unwrap().inner.clone()
                {
                    print!("{:}: ", unwrap_ident(&union_ident));
                }
            }
        }
        None => print!("{:}: ", unwrap_ident(ident)),
    }
}

/// Flatten `bitvector_concat` tree.
fn fold_bitvector_concat_tree(bitvec_concat: Expression) -> Vec<ExpressionAux> {
    if let ExpressionAux::Application(ident, exp_list) = *bitvec_concat.inner {
        // not leaf
        assert!(unwrap_ident(&ident).as_ref() == "bitvector_concat");
        [
            vec![*exp_list.iter().next().unwrap().inner.clone()],
            fold_bitvector_concat_tree(exp_list.iter().nth(1).unwrap().clone()),
        ]
        .iter()
        .flatten()
        .cloned()
        .collect()
    } else {
        // leaf
        vec![*bitvec_concat.inner.clone()]
    }
}

/// Show rhs of the encode data.
pub fn show_encoding_rule_rhs(pat_rhs: Expression) {
    let exp_list = fold_bitvector_concat_tree(pat_rhs);
    for exp_aux in exp_list {
        match exp_aux {
            ExpressionAux::Vector(list_exp) => {
                let bit_width = list_exp.len();
                let bit_vec = list_exp.iter().fold(0u32, |bits, exp| {
                    if let ExpressionAux::Literal(literal) = *exp.inner.clone() {
                        match literal.inner {
                            LiteralAux::Zero => bits.checked_shl(1).unwrap(),
                            LiteralAux::True => bits.checked_shl(1).unwrap() | 1,
                            _ => unreachable!(),
                        }
                    } else {
                        panic!("Vector element is not a literal")
                    }
                });

                print!("{bit_vec:0>bit_width$b} ");
            }
            ExpressionAux::Cast(typ, ident) => {
                // ident name
                let ExpressionAux::Identifier(cast_ident) = *ident.inner else {
                    panic!("unexpected ExpressionAux: {:#?}", *ident.inner);
                };

                // bit width
                let TypAux::Application(_ident, exp_list) = *typ.inner else {
                    panic!("unexpected TypAux: {:#?}", *typ.inner);
                };
                let typ_arg = exp_list.iter().next().unwrap();
                let TypArgAux::NExp(num_exp) = typ_arg.inner.clone() else {
                    panic!("unexpected TypArgAux: {:#?}", typ_arg.inner.clone());
                };
                let NumericExpressionAux::Constant(bit_width) = *num_exp.inner else {
                    panic!("unexpected NumericExpressionAux: {:#?}", *num_exp.inner);
                };

                print!("{}({} bit) ", unwrap_ident(&cast_ident), bit_width.0);
            }
            _ => unreachable!(),
        }
    }
    println!();
}

/// Get encode data of provided instructions.
pub fn show_encoding_rule(target_file_name: &str) {
    let encdec_node = AST.get().unwrap().get_encdec_forward_node().unwrap();
    let inst_list = get_insns_in_target_file(target_file_name);
    if let PatternMatchAux::Expression(_pat, exp) = encdec_node.inner.pattern_match.inner {
        if let ExpressionAux::Match(_exp, pat_list) = *exp.inner {
            for pat in pat_list {
                // pat_lhs: RISCV_SLLIUW(shamt, rs1, rd)
                // exp0: if extensionEnabled(Ext_Zba) & xlen == 64
                // pat_rhs: 0b000010 @ shamt @ rs1 @ 0b001 @ rd @ 0b0011011
                if let PatternMatchAux::When(pat_lhs, _exp0, pat_rhs) = pat.inner {
                    if let PatternAux::Application(ident, pat_list) = *pat_lhs.inner {
                        if let Some(inst) = inst_list
                            .iter()
                            .find(|x| x.name == unwrap_ident(&ident).as_ref())
                        {
                            show_encoding_rule_lhs(&ident, inst, &pat_list);
                            show_encoding_rule_rhs(pat_rhs.clone());
                        }
                    }
                }
            }
        }
    }
}
