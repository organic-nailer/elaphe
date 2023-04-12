use std::error::Error;

use super::{
    node::{FunctionSignature, Identifier, NodeStatement},
    node_internal::NodeInternal,
    parse_identifier::parse_identifier,
    parse_statement::parse_statement,
    parse_type::parse_type,
    util::{flatten, gen_error},
};

pub fn parse_function_body<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionBody" {
        if node.children.len() == 1 {
            return parse_statement(&node.children[0]);
        } else {
            return parse_statement(&node.children[1]);
        }
    }

    Err(gen_error("parse_function_body", &node.rule_name))
}

fn parse_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Identifier<'input>>, Box<dyn Error>> {
    if node.rule_name == "FormalParameterList" {
        if node.children.len() == 2 {
            return Ok(vec![]);
        } else {
            return parse_normal_formal_parameter_list(&node.children[1]);
        }
    }

    Err(gen_error("parse_formal_parameter_list", &node.rule_name))
}

fn parse_normal_formal_parameter_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Identifier<'input>>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameterList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_normal_formal_parameter(&node.children[0])?]);
        } else {
            return flatten(
                parse_normal_formal_parameter_list(&node.children[0]),
                parse_normal_formal_parameter(&node.children[2])?,
            );
        }
    }

    Err(gen_error(
        "parse_normal_formal_parameter_list",
        &node.rule_name,
    ))
}

fn parse_normal_formal_parameter<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "NormalFormalParameter" {
        return Ok(parse_identifier(&node.children[0])?);
    }

    Err(gen_error("parse_normal_formal_parameter", &node.rule_name))
}

pub fn parse_function_signature<'input>(
    node: &NodeInternal<'input>,
) -> Result<FunctionSignature<'input>, Box<dyn Error>> {
    if node.rule_name == "FunctionSignature" {
        if node.children.len() == 2 {
            return Ok(FunctionSignature {
                return_type: None,
                name: parse_identifier(&node.children[0])?,
                param: parse_formal_parameter_list(&node.children[1])?,
            });
        } else {
            return Ok(FunctionSignature {
                return_type: Some(parse_type(&node.children[0])?),
                name: parse_identifier(&node.children[1])?,
                param: parse_formal_parameter_list(&node.children[2])?,
            });
        }
    }

    Err(gen_error("parse_function_signature", &node.rule_name))
}
