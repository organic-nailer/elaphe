use std::error::Error;

use super::{
    node::{LibraryDeclaration, NodeStatement},
    node_internal::NodeInternal,
    parse_functions::{parse_function_body, parse_function_signature},
    parse_variables::parse_initialized_identifier_list,
    util::{flatten, gen_error},
};

pub fn parse_library<'input>(
    node: &NodeInternal<'input>,
) -> Result<LibraryDeclaration<'input>, Box<dyn Error>> {
    Ok(LibraryDeclaration {
        top_level_declaration_list: parse_top_level_declaration_list(&node.children[0])?,
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
            _ => {}
        }
    }

    Err(gen_error("parse_top_level_declaration", &node.rule_name))
}

fn parse_top_function_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "TopFunctionDeclaration" {
        return Ok(NodeStatement::FunctionDeclaration {
            signature: parse_function_signature(&node.children[0])?,
            body: Box::new(parse_function_body(&node.children[1])?),
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
