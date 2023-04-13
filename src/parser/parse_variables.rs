use std::error::Error;

use super::{
    node::VariableDeclaration,
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    parse_functions::parse_declared_identifier,
    parse_identifier::parse_identifier,
    util::{flatten, gen_error},
};

pub fn parse_initialized_variable_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<VariableDeclaration<'input>>, Box<dyn Error>> {
    if node.rule_name == "InitializedVariableDeclaration" {
        if node.children.len() == 1 {
            return Ok(vec![VariableDeclaration {
                identifier: parse_declared_identifier(&node.children[0])?,
                expr: None,
            }]);
        } else if node.children[1].token.clone().unwrap().str == "=" {
            return Ok(vec![VariableDeclaration {
                identifier: parse_declared_identifier(&node.children[0])?,
                expr: Some(Box::new(parse_expression(&node.children[2])?)),
            }]);
        } else {
            return flatten(
                parse_initialized_variable_declaration(&node.children[0]),
                parse_initialized_identifier(&node.children[2])?,
            );
        }
    }

    Err(gen_error(
        "parse_initialized_variable_declaration",
        &node.rule_name,
    ))
}

fn parse_initialized_identifier<'input>(
    node: &NodeInternal<'input>,
) -> Result<VariableDeclaration<'input>, Box<dyn Error>> {
    if node.rule_name == "InitializedIdentifier" {
        if node.children.len() == 1 {
            return Ok(VariableDeclaration {
                identifier: parse_identifier(&node.children[0])?,
                expr: None,
            });
        } else {
            return Ok(VariableDeclaration {
                identifier: parse_identifier(&node.children[0])?,
                expr: Some(Box::new(parse_expression(&node.children[2])?)),
            });
        }
    }

    Err(gen_error("parse_initialized_identifier", &node.rule_name))
}

pub fn parse_initialized_identifier_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<VariableDeclaration<'input>>, Box<dyn Error>> {
    if node.rule_name == "InitializedIdentifierList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_initialized_identifier(&node.children[0])?]);
        } else {
            return flatten(
                parse_initialized_identifier_list(&node.children[0]),
                parse_initialized_identifier(&node.children[2])?,
            );
        }
    }

    Err(gen_error(
        "parse_initialized_identifier_list",
        &node.rule_name,
    ))
}
