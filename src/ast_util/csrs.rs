use sailrs::sail_ast::{Expression, ExpressionAux, LiteralAux, PatternMatchAux};

use super::{check_defined_location, unwrap_ident};
use crate::AST;

/// Get csr number.
fn show_csr_number(csr_num: Expression) {
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
    print!("{csr_num:#05x} <-> ");
}

/// Get csr name.
fn show_csr_ident(csr_name: Expression) {
    let ExpressionAux::Literal(literal) = *csr_name.inner else {
        panic!("not a Literal");
    };
    let LiteralAux::String(name) = literal.inner else {
        panic!("not a String");
    };

    println!("{name}");
}

/// Get CSRs definitions.
pub fn show_csrs_definition(target_file_name: &str) {
    let csrs_node = AST.get().unwrap().get_csrs_forward_node().unwrap();
    if let PatternMatchAux::Expression(_pat, exp) = csrs_node.inner.pattern_match.inner {
        if let ExpressionAux::Match(_exp, pat_list) = *exp.inner {
            for pat in pat_list {
                // _pat_id: "b__0"
                // csr_num: "b__0" = 0bxxxx_xxxx_xxxx (12 bits)
                // pat_rhs: "fflags"
                if let PatternMatchAux::When(pat_id, csr_num, csr_ident) = pat.inner {
                    if check_defined_location(&pat_id.annotation, target_file_name) {
                        show_csr_number(csr_num);
                        show_csr_ident(csr_ident);
                    }
                }
            }
        }
    }
}
