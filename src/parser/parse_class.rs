use std::error::Error;

use super::{
    node::{DartType, Member, NodeStatement},
    node_internal::NodeInternal,
    parse_functions::{parse_function_body, parse_function_signature},
    parse_identifier::parse_identifier,
    parse_variables::parse_initialized_identifier_list,
    util::{flatten, gen_error},
};

pub fn parse_class_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    if node.rule_name == "ClassDeclaration" {
        if node.children.len() == 4 {
            return Ok(NodeStatement::ClassDeclaration {
                identifier: parse_identifier(&node.children[1])?,
                member_list: vec![],
            });
        } else {
            return Ok(NodeStatement::ClassDeclaration {
                identifier: parse_identifier(&node.children[1])?,
                member_list: parse_class_declaration_internal(&node.children[3])?,
            });
        }
    }

    Err(gen_error("parse_class_declaration", &node.rule_name))
}

fn parse_class_declaration_internal<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Member<'input>>, Box<dyn Error>> {
    if node.rule_name == "ClassDeclarationInternal" {
        if node.children.len() == 1 {
            return Ok(vec![parse_class_member_declaration(&node.children[0])?]);
        } else {
            return flatten(
                parse_class_declaration_internal(&node.children[0]),
                parse_class_member_declaration(&node.children[1])?,
            );
        }
    }

    Err(gen_error(
        "parse_class_declaration_internal",
        &node.rule_name,
    ))
}

fn parse_class_member_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<Member<'input>, Box<dyn Error>> {
    if node.rule_name == "ClassMemberDeclaration" {
        if node.children[0].rule_name == "Declaration" {
            return parse_declaration(&node.children[0]);
        } else {
            return parse_member_impl(&node.children[0]);
        }
    }

    Err(gen_error("parse_class_member_declaration", &node.rule_name))
}

fn parse_member_impl<'input>(
    node: &NodeInternal<'input>,
) -> Result<Member<'input>, Box<dyn Error>> {
    if node.rule_name == "MemberImpl" {
        let signature = parse_function_signature(&node.children[0])?;
        let return_is_void = match &signature.return_type {
            Some(return_type) => match return_type {
                DartType::Void => true,
                _ => false,
            },
            None => false,
        };
        return Ok(Member::MethodImpl {
            signature,
            body: Box::new(parse_function_body(&node.children[1], return_is_void)?),
        });
    }

    Err(gen_error("parse_member_impl", &node.rule_name))
}

fn parse_declaration<'input>(
    node: &NodeInternal<'input>,
) -> Result<Member<'input>, Box<dyn Error>> {
    if node.rule_name == "Declaration" {
        return Ok(Member::VariableDecl {
            decl_list: parse_initialized_identifier_list(node.children.last().unwrap())?,
        });
    }

    Err(gen_error("parse_declaration", &node.rule_name))
}
