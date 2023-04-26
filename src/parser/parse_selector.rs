use anyhow::{bail, Result};

use super::{
    node::{CallParameter, NodeExpression, Selector},
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    parse_identifier::parse_identifier,
    parse_statement::parse_label,
    util::flatten,
};

pub fn parse_slice_expression<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>> {
    if node.rule_name == "SliceExpression" {
        if node.children.len() == 3 {
            return Ok(NodeExpression::Slice {
                start: None,
                end: None,
                step: None,
            });
        } else if node.children.len() == 4 {
            return Ok(NodeExpression::Slice {
                start: Some(Box::new(parse_expression(&node.children[2])?)),
                end: None,
                step: None,
            });
        } else if node.children.len() == 6 {
            return Ok(NodeExpression::Slice {
                start: Some(Box::new(parse_expression(&node.children[2])?)),
                end: Some(Box::new(parse_expression(&node.children[4])?)),
                step: None,
            });
        } else if node.children.len() == 8 {
            return Ok(NodeExpression::Slice {
                start: Some(Box::new(parse_expression(&node.children[2])?)),
                end: Some(Box::new(parse_expression(&node.children[4])?)),
                step: Some(Box::new(parse_expression(&node.children[6])?)),
            });
        }
    }

    bail!("Parse Error in parse_slice_expression: {}", node.rule_name);
}

pub fn parse_selector<'input>(node: &NodeInternal<'input>) -> Result<Selector<'input>> {
    if node.rule_name == "Selector" {
        if node.children.len() == 1 {
            return Ok(Selector::Args {
                args: parse_arguments(&node.children[0])?,
            });
        } else if node.children.len() == 2 {
            return Ok(Selector::Attr {
                identifier: parse_identifier(&node.children[1])?,
            });
        } else if node.children.len() == 3 {
            if node.children[0].token.clone().unwrap().str == "." {
                return Ok(Selector::Method {
                    identifier: parse_identifier(&node.children[1])?,
                    arguments: parse_arguments(&node.children[2])?,
                });
            } else {
                return Ok(Selector::Index {
                    expr: Box::new(parse_expression(&node.children[1])?),
                });
            }
        }
    }

    bail!("Parse Error in parse_selector: {}", node.rule_name);
}

fn parse_arguments<'input>(node: &NodeInternal<'input>) -> Result<Vec<CallParameter<'input>>> {
    if node.rule_name == "Arguments" {
        if node.children.len() == 2 {
            return Ok(vec![]);
        } else if node.children.len() == 3 {
            return Ok(parse_argument_list(&node.children[1])?);
        }
    }

    bail!("Parse Error in parse_arguments: {}", node.rule_name);
}

fn parse_argument_item<'input>(node: &NodeInternal<'input>) -> Result<CallParameter<'input>> {
    if node.rule_name == "NormalArgument" {
        return Ok(CallParameter {
            identifier: None,
            expr: Box::new(parse_expression(&node.children[0])?),
        });
    } else if node.rule_name == "NamedArgument" {
        return Ok(CallParameter {
            identifier: Some(parse_label(&node.children[0])?),
            expr: Box::new(parse_expression(&node.children[1])?),
        });
    }

    bail!("Parse Error in parse_argument_item: {}", node.rule_name);
}

fn parse_argument_list<'input>(node: &NodeInternal<'input>) -> Result<Vec<CallParameter<'input>>> {
    if node.rule_name == "ArgumentList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_argument_item(&node.children[0])?]);
        } else {
            return flatten(
                parse_argument_list(&node.children[0]),
                parse_argument_item(&node.children[2])?,
            );
        }
    }

    bail!("Parse Error in parse_argument_list: {}", node.rule_name);
}
