use std::path::Path;

use log::info;

use sailrs::jib_ast;
use sailrs::jib_ast::{DefinitionAux, TypeDefinition};
use sailrs::sail_ast::{Identifier, IdentifierAux, Location};
use sailrs::types::ListVec;

/// find node included target file.
#[allow(dead_code)]
fn node_in_target_file(jib: ListVec<jib_ast::Definition>, target_file_name: &str) {
    let mut node_count = 0;
    for def in jib {
        if let Some(generated) = def.annot.attrs.iter().next() {
            if let Location::Generated(range) = &generated.0 {
                if let Location::Range(pos, _) = range.as_ref() {
                    let contain_file = Path::new(pos.pos_fname.as_ref()).file_name().unwrap();
                    if contain_file == target_file_name {
                        node_count += 1;
                        match def.def {
                            DefinitionAux::Register(ident, _, _)
                            | DefinitionAux::Fundef(ident, _, _, _) => {
                                println!("{:?}", ident.inner);
                            }
                            DefinitionAux::Type(type_def) => println!("{type_def:#?}"),
                            DefinitionAux::Let(_int, _ident_typ_list, _inst_list) => todo!(),
                            DefinitionAux::Val(_ident, _, _type_list, _typ) => todo!(),
                            DefinitionAux::Startup(_ident, _inst_list) => todo!(),
                            DefinitionAux::Finish(_ident, _inst_list) => todo!(),
                            DefinitionAux::Pragma(_, _) => todo!(),
                        }
                    }
                }
            }
        }
    }
    info!("Number of node: {node_count}");
}

/// Show all nodes.
#[allow(dead_code)]
fn show_all_nodes(jib: ListVec<jib_ast::Definition>) {
    let mut node_count = 0;
    for def in jib {
        node_count += 1;
        match def.def {
            DefinitionAux::Val(ident, _, _, _)
            | DefinitionAux::Register(ident, _, _)
            | DefinitionAux::Fundef(ident, _, _, _) => {
                println!("{:?}", ident.inner);
            }
            DefinitionAux::Type(type_def) => println!("{type_def:#?}"),
            DefinitionAux::Let(_int, ident_typ_list, _inst_list) => {
                println!("{ident_typ_list:#?}");
            }
            DefinitionAux::Startup(_ident, _inst_list) => todo!(),
            DefinitionAux::Finish(_ident, _inst_list) => todo!(),
            DefinitionAux::Pragma(interned_str, _) => println!("{interned_str:#?}"),
        }
    }
    info!("Number of node: {node_count}");
}

/// Get node of "ast" variant.
#[allow(dead_code)]
pub fn get_ast_node(jib: ListVec<jib_ast::Definition>) -> Result<jib_ast::Definition, ()> {
    for def in jib {
        if let DefinitionAux::Type(TypeDefinition::Variant(ref ident, ref _list)) = def.def {
            if let IdentifierAux::Identifier(ident_str) = ident.inner {
                if ident_str == "ast".into() {
                    return Ok(def);
                }
            }
        }
    }

    Err(())
}

/// Is the ident in target file?
#[allow(dead_code)]
fn is_ident_in_target_file(ident: &Identifier, target_file_name: &str) -> bool {
    if let Location::Range(ref pos, _) = ident.location {
        let contain_file = Path::new(pos.pos_fname.as_ref()).file_name().unwrap();
        contain_file == target_file_name
    } else {
        false
    }
}

/// Show all Instruction in target file.
#[allow(dead_code)]
pub fn list_inst(ast_node: jib_ast::Definition, target_file_name: &str) {
    if let DefinitionAux::Type(TypeDefinition::Variant(_ast_ident, list)) = ast_node.def {
        for (union_ident, child) in list {
            if is_ident_in_target_file(&union_ident, target_file_name) {
                println!("{:#?}", union_ident.inner);
                println!("{child:#?}");
            }
        }
    }
}
