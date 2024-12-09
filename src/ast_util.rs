pub mod csrs;
pub mod instruction;

use std::path::Path;
use std::sync::OnceLock;

use common::intern::InternedString;
use sailrs::sail_ast;
use sailrs::sail_ast::{
    Annot, DefinitionAux, Identifier, IdentifierAux, Location, TypeDefinitionAux,
};
use sailrs::types::ListVec;

/// Abstract Syntex Tree
pub static AST: OnceLock<Ast> = OnceLock::new();

/// Wrapper of `sail_ast::Ast`.
#[derive(Debug)]
pub struct Ast(sail_ast::Ast);

impl Ast {
    /// Create itself.
    pub fn new(ast: sail_ast::Ast) -> Self {
        Self(ast)
    }

    /// Get node of "ast" variant.
    pub fn get_ast_node(&self) -> Result<sail_ast::Definition, ()> {
        for def in self.0.clone().defs {
            if let DefinitionAux::Type(ref type_def) = def.definition {
                if let TypeDefinitionAux::Variant(ident, _list, _union_list, _flag) =
                    &type_def.inner
                {
                    if let IdentifierAux::Identifier(ident_str) = ident.inner {
                        if ident_str == "ast".into() {
                            return Ok(def);
                        }
                    }
                }
            }
        }

        Err(())
    }

    /// Get node of `csr_name_map_forwards` definition.
    pub fn get_csrs_forward_node(&self) -> Result<sail_ast::FunctionClause, ()> {
        for def in self.0.clone().defs {
            if let DefinitionAux::Function(ref func_def) = def.definition {
                let func_clause = &func_def.inner.clauses.iter().nth(0).unwrap();
                let func_clause_ident = func_clause.inner.identifier.inner.clone();
                if let IdentifierAux::Identifier(ident) = func_clause_ident {
                    if ident == "csr_name_map_forwards".into() {
                        return Ok((*func_clause).clone());
                    }
                }
            }
        }

        Err(())
    }

    /// Get node of `encdec_forward` definition.
    pub fn get_encdec_forward_node(&self) -> Result<sail_ast::FunctionClause, ()> {
        for def in self.0.clone().defs {
            if let DefinitionAux::Function(ref func_def) = def.definition {
                let func_clause = &func_def.inner.clauses.iter().nth(0).unwrap();
                let func_clause_ident = func_clause.inner.identifier.inner.clone();
                if let IdentifierAux::Identifier(ident) = func_clause_ident {
                    if ident == "encdec_forwards".into() {
                        return Ok((*func_clause).clone());
                    }
                }
            }
        }

        Err(())
    }

    /// Find an enum definition by name
    fn find_enum(&self, ident_name: &IdentifierAux) -> Option<sail_ast::TypeDefinition> {
        if let IdentifierAux::Identifier(ident_name) = ident_name {
            for def in self.0.clone().defs {
                if let DefinitionAux::Type(ref type_def) = def.definition {
                    if let TypeDefinitionAux::Enumeration(ident, _list, _flag) = &type_def.inner {
                        if let IdentifierAux::Identifier(ident_str) = ident.inner {
                            if ident_str == *ident_name {
                                return Some(type_def.clone());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get enum variants by name
    pub fn get_enum_variants(&self, ident_name: &IdentifierAux) -> Option<ListVec<Identifier>> {
        let found_enum = self.find_enum(ident_name)?;
        if let TypeDefinitionAux::Enumeration(_ident, list, _flag) = found_enum.inner {
            Some(list)
        } else {
            None
        }
    }
}

/// unwrap `IdentifierAux::Identifier(T)` to `T`.
fn unwrap_ident(ident: &Identifier) -> InternedString {
    if let IdentifierAux::Identifier(ident) = ident.inner {
        ident
    } else {
        panic!("not an identifier")
    }
}

/// Is it defined in target file?
fn check_defined_location(annot: &Annot, target_file_name: &str) -> bool {
    match &annot.location {
        Location::Range(begin, _end) => {
            let contain_file = Path::new(begin.pos_fname.as_ref()).file_name().unwrap();
            contain_file == target_file_name
        }
        Location::Generated(_annot) => todo!("TODO: implement generated location"),
        Location::Unique(..) | Location::Hint(..) => unimplemented!(),
        Location::Unknown => panic!("Unknown position"),
    }
}
