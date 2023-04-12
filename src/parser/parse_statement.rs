use std::error::Error;

use super::{
    node::NodeStatement,
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    util::{flatten, gen_error},
};

pub fn parse_statement<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeStatement<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "Statement" => parse_statement(&node.children[0]),
        "BlockStatement" => Ok(NodeStatement::BlockStatement {
            statements: parse_statement_list(&node.children[1])?,
        }),
        "ExpressionStatement" => Ok(NodeStatement::ExpressionStatement {
            expr: Box::new(parse_expression(&node.children[0])?),
        }),
        v => Err(gen_error("parse_statement", v)),
    }
}

pub fn parse_statement_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<NodeStatement<'input>>, Box<dyn Error>> {
    if node.rule_name == "Statements" {
        if node.children.len() == 0 {
            return Ok(vec![]);
        } else {
            return flatten(
                parse_statement_list(&node.children[0]),
                parse_statement(&node.children[1])?,
            );
        }
    }

    Err(gen_error("parse_statement_list", &node.rule_name))
}
