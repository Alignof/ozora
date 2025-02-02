use std::ops::Range;
use std::path::Path;

use rand::Rng;
use sailrs::sail_ast::{
    DefinitionAux, Expression, ExpressionAux, Identifier, IdentifierAux, LiteralAux, Location,
    NumericExpressionAux, Pattern, PatternAux, PatternMatchAux, TypArgAux, TypAux,
    TypeDefinitionAux, TypeUnion,
};
use sailrs::types::ListVec;

use super::unwrap_ident;
use crate::{AST, XLEN};

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

/// Operand data
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Operand {
    /// Field name
    name: String,
    /// Bit map of the field.
    pub range: Range<u8>,
}

/// Opecode data
#[derive(Debug, Clone)]
pub struct Opecode {
    /// Opecode value
    pub value: u32,
    /// Bit map of the field.
    pub range: Range<u8>,
}

/// Bit field kind.
#[derive(Debug, Clone)]
pub enum Field {
    /// Opecode value
    Opc(Opecode),
    /// Opr field. (e.g. `rd`, `rs1`)
    Opr(Operand),
}

/// Instruction data
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Instruction name.
    pub name: String,

    /// Group name (Name of `InstType`)
    _group_name: Option<String>,

    /// List of operands.
    fields: Vec<Field>,
}

impl Instruction {
    /// Get a field by name
    pub fn get_field_by_name(&self, field_name: &str) -> Option<&Operand> {
        self.fields.iter().find_map(|x| match x {
            Field::Opr(n) => {
                if n.name == field_name {
                    Some(n)
                } else {
                    None
                }
            }
            Field::Opc(_) => None,
        })
    }

    /// Get all opecode fields
    pub fn get_opc_fields(&self) -> Vec<&Opecode> {
        self.fields
            .iter()
            .filter_map(|x| match x {
                Field::Opc(opc) => Some(opc),
                Field::Opr(_) => None,
            })
            .collect()
    }

    /// Get opecode value by range
    pub fn get_opc_value_by_range(&self, range: &Range<u8>) -> Option<u32> {
        self.fields.iter().find_map(|x| match x {
            Field::Opc(opc) => {
                if opc.range == *range {
                    Some(opc.value)
                } else {
                    None
                }
            }
            Field::Opr(_) => None,
        })
    }

    /// Get random instruction value.
    ///
    /// This function mainly targets for unit test.
    #[allow(clippy::cast_sign_loss)]
    pub fn get_random_insn_value(
        &self,
    ) -> (u32, Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
        let mut rng = rand::thread_rng();
        let rd = self
            .get_field_by_name("rd")
            .map(|x| rng.gen_range(0..(1 << x.range.len())) as u32);
        let rs1 = self
            .get_field_by_name("rs1")
            .map(|x| rng.gen_range(0..(1 << x.range.len())) as u32);
        let rs2 = self
            .get_field_by_name("rs2")
            .map(|x| rng.gen_range(0..(1 << x.range.len())) as u32);
        let mut imm = self
            .get_field_by_name("imm")
            .map(|x| rng.gen_range(0..(1 << x.range.len())) as u32);

        let insn_val = self.fields.iter().fold(0u32, |bits, field| match field {
            Field::Opr(opr) => {
                bits << opr.range.len()
                    | match opr.name.as_str() {
                        "rd" => rd.unwrap(),
                        "rs1" => rs1.unwrap(),
                        "rs2" => rs2.unwrap(),
                        "imm" => imm.unwrap(),
                        "shamt" => {
                            let new_rand = rng.gen_range(0..(1 << opr.range.len())) as u32;
                            imm = Some(new_rand);
                            new_rand
                        }
                        _ => panic!("unsupported operand: {}", opr.name),
                    }
            }
            Field::Opc(opc) => bits << opc.range.len() | opc.value,
        });

        (insn_val, rd, rs1, rs2, imm)
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
pub fn get_encoding_rule_lhs(
    ident: &Identifier,
    inst: &InstType,
    pat_list: &ListVec<Pattern>,
) -> String {
    match inst.index {
        Some(index) => {
            if let PatternAux::Tuple(union_args) = *pat_list.iter().next().unwrap().inner.clone() {
                if let PatternAux::Identifier(union_ident) =
                    *union_args.iter().nth(index).unwrap().inner.clone()
                {
                    return unwrap_ident(&union_ident).to_string();
                }
            }

            panic!("couldn't get a instruction name");
        }
        None => unwrap_ident(ident).to_string(),
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

/// Get rhs of the encode data.
#[allow(clippy::cast_possible_truncation)]
pub fn get_encoding_rule_rhs(pat_rhs: Expression) -> Vec<Field> {
    let mut op_list = Vec::new();
    let mut offset = 0;
    let exp_list = fold_bitvector_concat_tree(pat_rhs);

    for exp_aux in exp_list.iter().rev() {
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

                op_list.push(Field::Opc(Opecode {
                    value: bit_vec,
                    range: offset..u8::try_from(bit_width).unwrap() - 1 + offset,
                }));

                offset += bit_width as u8;
            }
            ExpressionAux::Cast(typ, ident) => {
                // ident name
                let ExpressionAux::Identifier(ref cast_ident) = *ident.inner else {
                    panic!("unexpected ExpressionAux: {:#?}", *ident.inner);
                };

                // bit width
                let TypAux::Application(ref _ident, ref exp_list) = *typ.inner else {
                    panic!("unexpected TypAux: {:#?}", *typ.inner);
                };
                let typ_arg = exp_list.iter().next().unwrap();
                let TypArgAux::NExp(num_exp) = typ_arg.inner.clone() else {
                    panic!("unexpected TypArgAux: {:#?}", typ_arg.inner.clone());
                };
                let NumericExpressionAux::Constant(bit_width) = *num_exp.inner else {
                    panic!("unexpected NumericExpressionAux: {:#?}", *num_exp.inner);
                };

                op_list.push(Field::Opr(Operand {
                    name: unwrap_ident(cast_ident).to_string(),
                    range: offset..u8::try_from(bit_width.0.clone()).unwrap() - 1 + offset,
                }));

                offset += u8::try_from(bit_width.0).unwrap();
            }
            _ => unreachable!(),
        }
    }

    op_list
}

/// Get encode data of provided instructions.
pub fn get_encoding_rule(target_file_name: &str) -> Vec<Instruction> {
    let encdec_node = AST.get().unwrap().get_encdec_forward_node().unwrap();
    let mut inst_list = Vec::new();
    let inst_type_list = get_insns_in_target_file(target_file_name);

    if let PatternMatchAux::Expression(_pat, exp) = encdec_node.inner.pattern_match.inner {
        if let ExpressionAux::Match(_exp, pat_list) = *exp.inner {
            for pat in pat_list {
                // pat_lhs: RISCV_SLLIUW(shamt, rs1, rd)
                // exp0: if extensionEnabled(Ext_Zba) & xlen == 64
                // pat_rhs: 0b000010 @ shamt @ rs1 @ 0b001 @ rd @ 0b0011011
                if let PatternMatchAux::When(pat_lhs, ref exp0, pat_rhs) = pat.inner {
                    dbg!(exp0);
                    if let ExpressionAux::Application(ref _ident, ref pat_list) = *exp0.inner {
                        assert_eq!(pat_list.len(), 2);
                        if let ExpressionAux::Application(_ident, ref lhs_rhs) =
                            *pat_list.iter().nth(1).unwrap().inner.clone()
                        {
                            let ExpressionAux::Identifier(xlen_name) =
                                *lhs_rhs.iter().nth(0).unwrap().inner.clone()
                            else {
                                panic!();
                            };

                            let IdentifierAux::Identifier(xlen_str) = xlen_name.inner else {
                                panic!();
                            };
                            assert_eq!(xlen_str, "xlen".into());

                            let ExpressionAux::Literal(xlen_val) =
                                *lhs_rhs.iter().nth(1).unwrap().inner.clone()
                            else {
                                panic!();
                            };

                            if let LiteralAux::Num(xlen) = xlen_val.inner {
                                if xlen != sailrs::num::BigInt(XLEN.into()) {
                                    continue;
                                }
                            }
                        }
                    }
                    if let PatternAux::Application(ident, pat_list) = *pat_lhs.inner {
                        if let Some(inst) = inst_type_list
                            .iter()
                            .find(|x| x.name == unwrap_ident(&ident).as_ref())
                        {
                            inst_list.push(Instruction {
                                name: get_encoding_rule_lhs(&ident, inst, &pat_list),
                                _group_name: inst.index.map(|_| inst.name.clone()),
                                fields: get_encoding_rule_rhs(pat_rhs.clone()),
                            });
                        }
                    }
                }
            }
        }
    }

    inst_list
}
