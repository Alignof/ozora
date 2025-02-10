use sailrs::sail_ast::{Expression, ExpressionAux, LiteralAux, PatternMatchAux};

use super::{check_defined_location, unwrap_ident};
use crate::AST;

/// CSR number
#[derive(Debug)]
struct CsrNumber(u32);

/// CSR data
#[derive(Debug)]
pub struct Csr {
    /// CSR name
    name: String,
    /// CSR number
    number: CsrNumber,
}

/// Get csr number.
fn get_csr_number(csr_num: Expression) -> CsrNumber {
    let ExpressionAux::Application(ident, def_tuple) = *csr_num.inner else {
        panic!("not a ExpressionAux");
    };

    assert_eq!(unwrap_ident(&ident).as_ref(), "eq_bits");
    assert_eq!(def_tuple.len(), 2);
    let bit_vec = def_tuple.iter().nth(1).unwrap();
    let ExpressionAux::Vector(ref bit_vec) = *bit_vec.inner else {
        panic!("eq_bits does not containt vector");
    };
    let csr_num = bit_vec.iter().fold(0u32, |bits, exp| {
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

    CsrNumber(csr_num)
}

/// Get csr name.
fn get_csr_ident(csr_name: Expression) -> String {
    let ExpressionAux::Literal(literal) = *csr_name.inner else {
        panic!("not a Literal");
    };
    let LiteralAux::String(name) = literal.inner else {
        panic!("not a String");
    };

    name.to_string()
}

/// Get CSRs definitions.
pub fn get_csrs_definition(target_file_name: &str) -> Vec<Csr> {
    let mut csrs = Vec::new();
    let csrs_node = AST.get().unwrap().get_csrs_forward_node().unwrap();
    if let PatternMatchAux::Expression(_pat, exp) = csrs_node.inner.pattern_match.inner {
        if let ExpressionAux::Match(_exp, pat_list) = *exp.inner {
            for pat in pat_list {
                // _pat_id: "b__0"
                // csr_num: "b__0" = 0bxxxx_xxxx_xxxx (12 bits)
                // pat_rhs: "fflags"
                if let PatternMatchAux::When(pat_id, csr_num, csr_ident) = pat.inner {
                    if check_defined_location(&pat_id.annotation, target_file_name) {
                        csrs.push(Csr {
                            name: get_csr_ident(csr_ident),
                            number: get_csr_number(csr_num),
                        })
                    }
                }
            }
        }
    }

    csrs
}
