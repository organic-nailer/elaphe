use std::{error::Error, vec};

use super::{
    node::{Combinator, DartType, LibraryDeclaration, LibraryImport, NodeStatement},
    node_internal::NodeInternal,
    parse_class::parse_class_declaration,
    parse_functions::{parse_function_body, parse_function_signature},
    parse_identifier::{parse_identifier, parse_identifier_list},
    parse_variables::parse_initialized_identifier_list,
    util::{flatten, gen_error},
};

pub fn parse_library<'input>(
    node: &NodeInternal<'input>,
) -> Result<LibraryDeclaration<'input>, Box<dyn Error>> {
    Ok(LibraryDeclaration {
        import_list: parse_library_import_list(&node.children[0])?,
        top_level_declaration_list: parse_top_level_declaration_list(&node.children[1])?,
    })
}

fn parse_top_level_declaration_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Box<NodeStatement<'input>>>, Box<dyn Error>> {
    if node.rule_name == "TopLevelDeclarationList" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_top_level_declaration_list(&node.children[0]),
                Box::new(parse_top_level_declaration(&node.children[1])?),
            );
        }
    }

    Err(gen_error(
        "parse_top_level_declaration_list",
        &node.rule_name,
    ))
}

fn parse_top_level_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopLevelDeclaration" {
        match node.children[0].rule_name.as_str() {
            "TopFunctionDeclaration" => {
                return Ok(parse_top_function_declaration(&node.children[0])?);
            }
            "TopVariableDeclaration" => {
                return Ok(parse_top_variable_declaration(&node.children[0])?);
            }
            "ClassDeclaration" => {
                return Ok(parse_class_declaration(&node.children[0])?);
            }
            _ => {}
        }
    }

    Err(gen_error("parse_top_level_declaration", &node.rule_name))
}

fn parse_library_import_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<LibraryImport<'input>>, Box<dyn Error>> {
    if node.rule_name == "LibraryImportList" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_library_import_list(&node.children[0]),
                parse_library_import(&node.children[1])?,
            );
        }
    }

    Err(gen_error("parse_library_import_list", &node.rule_name))
}

fn parse_library_import<'input>(
    node: &NodeInternal<'input>,
) -> Result<LibraryImport<'input>, Box<dyn Error>> {
    if node.rule_name == "LibraryImport" {
        if node.children.len() == 3 {
            return Ok(LibraryImport {
                uri: node.children[1].children[0].token.clone().unwrap().str,
                identifier: None,
                combinator_list: vec![],
            });
        } else if node.children.len() == 4 {
            return Ok(LibraryImport {
                uri: node.children[1].children[0].token.clone().unwrap().str,
                identifier: None,
                combinator_list: parse_combinator_list(&node.children[2])?,
            });
        } else if node.children.len() == 5 {
            return Ok(LibraryImport {
                uri: node.children[1].children[0].token.clone().unwrap().str,
                identifier: Some(parse_identifier(&node.children[3])?),
                combinator_list: vec![],
            });
        } else if node.children.len() == 6 {
            return Ok(LibraryImport {
                uri: node.children[1].children[0].token.clone().unwrap().str,
                identifier: Some(parse_identifier(&node.children[3])?),
                combinator_list: parse_combinator_list(&node.children[4])?,
            });
        }
    }

    Err(gen_error("parse_library_import", &node.rule_name))
}

fn parse_top_function_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopFunctionDeclaration" {
        let signature = parse_function_signature(&node.children[0])?;
        let return_is_void = match &signature.return_type {
            Some(return_type) => match return_type {
                DartType::Void => true,
                _ => false,
            },
            None => false,
        };
        return Ok(NodeStatement::FunctionDeclaration {
            signature: parse_function_signature(&node.children[0])?,
            body: Box::new(parse_function_body(&node.children[1], return_is_void)?),
        });
    }

    Err(gen_error("parse_top_function_declaration", &node.rule_name))
}

fn parse_top_variable_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopVariableDeclaration" {
        if node.children.len() == 3 {
            return Ok(NodeStatement::VariableDeclarationList {
                decl_list: parse_initialized_identifier_list(&node.children[1])?,
            });
        } else if node.children.len() == 4 {
            return Ok(NodeStatement::VariableDeclarationList {
                decl_list: parse_initialized_identifier_list(&node.children[2])?,
            });
        } else if node.children.len() == 5 {
            return Ok(NodeStatement::VariableDeclarationList {
                decl_list: parse_initialized_identifier_list(&node.children[3])?,
            });
        }
    }

    Err(gen_error("parse_top_variable_declaration", &node.rule_name))
}

fn parse_combinator_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Combinator<'input>>, Box<dyn Error>> {
    if node.rule_name == "CombinatorList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_combinator(&node.children[0])?]);
        } else {
            return flatten(
                parse_combinator_list(&node.children[0]),
                parse_combinator(&node.children[1])?,
            );
        }
    }

    Err(gen_error("parse_combinator_list", &node.rule_name))
}

fn parse_combinator<'input>(
    node: &NodeInternal<'input>,
) -> Result<Combinator<'input>, Box<dyn Error>> {
    if node.rule_name == "Combinator" {
        if node.children[0].rule_name == "show" {
            return Ok(Combinator {
                is_show: true,
                target_list: parse_identifier_list(&node.children[1])?,
            });
        } else if node.children[0].rule_name == "hide" {
            return Ok(Combinator {
                is_show: false,
                target_list: parse_identifier_list(&node.children[1])?,
            });
        }
    }

    Err(gen_error("parse_combinator", &node.rule_name))
}
