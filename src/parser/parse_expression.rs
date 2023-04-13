use std::error::Error;

use super::{
    node::{CallParameter, NodeExpression, Selector},
    node_internal::NodeInternal,
    parse_identifier::parse_identifier,
    util::{flatten, gen_error},
};

pub fn parse_expression<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "Expression" => parse_expression(&node.children[0]),
        "PrimaryExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                parse_expression(&node.children[1])
            }
        }
        "Boolean" => Ok(NodeExpression::BooleanLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "Number" => Ok(NodeExpression::NumericLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "StringLiteralList" => Ok(NodeExpression::StringLiteral {
            str_list: parse_string_literal_list(&node)?,
        }),
        "Identifier" => Ok(NodeExpression::Identifier {
            identifier: parse_identifier(node)?,
        }),
        "ConditionalExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let condition = parse_expression(&node.children[0])?;
                let then = parse_expression(&node.children[2])?;
                let otherwise = parse_expression(&node.children[4])?;
                Ok(NodeExpression::Conditional {
                    condition: Box::new(condition),
                    true_expr: Box::new(then),
                    false_expr: Box::new(otherwise),
                })
            }
        }
        "IfNullExpression"
        | "LogicalOrExpression"
        | "LogicalAndExpression"
        | "BitwiseOrExpression"
        | "BitwiseXorExpression"
        | "BitwiseAndExpression"
        | "AdditiveExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                let right = parse_expression(&node.children[2])?;
                Ok(NodeExpression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator: &node.children[1].token.clone().unwrap().str,
                })
            }
        }
        "EqualityExpression"
        | "RelationalExpression"
        | "ShiftExpression"
        | "MultiplicativeExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                let right = parse_expression(&node.children[2])?;
                let operator = &node.children[1].children[0].token.clone().unwrap().str;
                Ok(NodeExpression::Binary {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                })
            }
        }
        "UnaryExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let expr = parse_expression(&node.children[1])?;
                let operator = &node.children[0].children[0].token.clone().unwrap().str;
                Ok(NodeExpression::Unary {
                    expr: Box::new(expr),
                    operator,
                })
            }
        }
        "PostfixExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                Ok(NodeExpression::Selector {
                    left: Box::new(left),
                    operator: parse_selector(&node.children[1])?,
                })
            }
        }
        v => Err(gen_error("parse_expression", v)),
    }
}

pub fn parse_expression_opt<'input>(
    node: &NodeInternal<'input>,
) -> Result<Option<Box<NodeExpression<'input>>>, Box<dyn Error>> {
    if node.rule_name == "ExpressionOpt" {
        if node.children.len() == 0 {
            return Ok(None);
        } else {
            return Ok(Some(Box::new(parse_expression(&node.children[0])?)));
        }
    }

    Err(gen_error("parse_expression_opt", &node.rule_name))
}

pub fn parse_expression_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Box<NodeExpression<'input>>>, Box<dyn Error>> {
    if node.rule_name == "ExpressionList" {
        if node.children.len() == 1 {
            return Ok(vec![Box::new(parse_expression(&node.children[0])?)]);
        } else {
            return flatten(
                parse_expression_list(&node.children[0]),
                Box::new(parse_expression(&node.children[1])?),
            );
        }
    }

    Err(gen_error("parse_expression_list", &node.rule_name))
}

pub fn parse_expression_list_opt<'input>(
    node: &NodeInternal<'input>,
) -> Result<Option<Vec<Box<NodeExpression<'input>>>>, Box<dyn Error>> {
    if node.rule_name == "ExpressionListOpt" {
        if node.children.len() == 0 {
            return Ok(None);
        } else {
            return Ok(Some(parse_expression_list(&node.children[0])?));
        }
    }

    Err(gen_error("parse_expression_list_opt", &node.rule_name))
}

fn parse_string_literal_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<&'input str>, Box<dyn Error>> {
    if node.rule_name == "StringLiteralList" {
        if node.children.len() == 1 {
            return Ok(vec![&node.children[0].token.clone().unwrap().str]);
        } else {
            return flatten(
                parse_string_literal_list(&node.children[0]),
                &node.children[1].token.clone().unwrap().str,
            );
        }
    }

    Err(gen_error("parse_string_literal_list", &node.rule_name))
}

fn parse_selector<'input>(node: &NodeInternal<'input>) -> Result<Selector<'input>, Box<dyn Error>> {
    if node.rule_name != "Selector" {
        return Err(gen_error("parse_selector", &node.rule_name));
    }
    let node = &node.children[0];
    match node.rule_name.as_str() {
        "Arguments" => {
            if node.children.len() == 2 {
                Ok(Selector::Args { args: Vec::new() })
            } else {
                Ok(Selector::Args {
                    args: parse_argument_list(&node.children[1])?,
                })
            }
        }
        v => Err(gen_error("parse_selector", v)),
    }
}

fn parse_normal_argument<'input>(
    node: &NodeInternal<'input>,
) -> Result<CallParameter<'input>, Box<dyn Error>> {
    if node.rule_name == "NormalArgument" {
        return Ok(CallParameter {
            identifier: None,
            expr: Box::new(parse_expression(&node.children[0])?),
        });
    }

    Err(gen_error("parse_normal_argument", &node.rule_name))
}

fn parse_argument_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<CallParameter<'input>>, Box<dyn Error>> {
    if node.rule_name == "ArgumentList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_normal_argument(&node.children[0])?]);
        } else {
            return flatten(
                parse_argument_list(&node.children[0]),
                parse_normal_argument(&node.children[2])?,
            );
        }
    }

    Err(gen_error("parse_argument_list", &node.rule_name))
}