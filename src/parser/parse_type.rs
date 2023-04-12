use std::error::Error;

use super::{
    node::{DartType, DartTypeName},
    node_internal::NodeInternal,
    parse_identifier::parse_identifier,
    util::gen_error,
};

pub fn parse_type<'input>(node: &NodeInternal<'input>) -> Result<DartType<'input>, Box<dyn Error>> {
    if node.rule_name == "Type" {
        return parse_type_not_function(&node.children[0]);
    }

    Err(gen_error("parse_type", &node.rule_name))
}

fn parse_type_not_function<'input>(
    node: &NodeInternal<'input>,
) -> Result<DartType<'input>, Box<dyn Error>> {
    if node.rule_name == "TypeNotFunction" {
        if node.children[0].token.clone().unwrap().str == "void" {
            return Ok(DartType::Void);
        } else {
            return parse_type_not_void_not_function(&node.children[0]);
        }
    }

    Err(gen_error("parse_type_not_function", &node.rule_name))
}

fn parse_type_not_void_not_function<'input>(
    node: &NodeInternal<'input>,
) -> Result<DartType<'input>, Box<dyn Error>> {
    if node.rule_name == "TypeNotVoidNotFunction" {
        return Ok(DartType::Named {
            type_name: parse_type_name(&node.children[0])?,
            type_arguments: vec![],
            is_nullable: false,
        });
    }

    Err(gen_error(
        "parse_type_not_void_not_function",
        &node.rule_name,
    ))
}

fn parse_type_name<'input>(
    node: &NodeInternal<'input>,
) -> Result<DartTypeName<'input>, Box<dyn Error>> {
    if node.rule_name == "TypeName" {
        return Ok(DartTypeName {
            identifier: parse_identifier(&node.children[0])?,
            module: None,
        });
    }

    Err(gen_error("parse_type_name", &node.rule_name))
}
