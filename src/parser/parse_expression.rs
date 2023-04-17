use std::error::Error;

use super::{
    node::NodeExpression,
    node_internal::NodeInternal,
    parse_identifier::parse_identifier,
    parse_literal::{parse_list_literal, parse_set_or_map_literal, parse_string_literal_list},
    parse_selector::{parse_selector, parse_slice_expression},
    util::{flatten, gen_error},
};

pub fn parse_expression<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "Expression" | "ExpressionNotBrace" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                let operator = &node.children[1].children[0].token.clone().unwrap().str;
                let right = parse_expression(&node.children[2])?;
                Ok(NodeExpression::Assignment {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
        }
        "PrimaryExpression" | "PrimaryExpressionNotBrace" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                parse_expression(&node.children[1])
            }
        }
        "NULL" => Ok(NodeExpression::NullLiteral),
        "BOOLEAN" => Ok(NodeExpression::BooleanLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "NUMBER" => Ok(NodeExpression::NumericLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "ThisExpression" => Ok(NodeExpression::This),
        "StringLiteralList" => Ok(NodeExpression::StringLiteral {
            str_list: parse_string_literal_list(&node)?,
        }),
        "Identifier" => Ok(NodeExpression::Identifier {
            identifier: parse_identifier(node)?,
        }),
        "ListLiteral" => parse_list_literal(node),
        "SetOrMapLiteral" => parse_set_or_map_literal(node),
        "ConditionalExpression" | "ConditionalExpressionNotBrace" => {
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
        | "IfNullExpressionNotBrace"
        | "LogicalOrExpression"
        | "LogicalOrExpressionNotBrace"
        | "LogicalAndExpression"
        | "LogicalAndExpressionNotBrace"
        | "BitwiseOrExpression"
        | "BitwiseOrExpressionNotBrace"
        | "BitwiseXorExpression"
        | "BitwiseXorExpressionNotBrace"
        | "BitwiseAndExpression"
        | "BitwiseAndExpressionNotBrace"
        | "AdditiveExpression"
        | "AdditiveExpressionNotBrace" => {
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
        | "EqualityExpressionNotBrace"
        | "RelationalExpression"
        | "RelationalExpressionNotBrace"
        | "ShiftExpression"
        | "ShiftExpressionNotBrace"
        | "MultiplicativeExpression"
        | "MultiplicativeExpressionNotBrace" => {
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
        "UnaryExpression" | "UnaryExpressionNotBrace" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                if node.children[0].rule_name == "PrefixOperator" {
                    let expr = parse_expression(&node.children[1])?;
                    let operator = &node.children[0].children[0].token.clone().unwrap().str;
                    Ok(NodeExpression::Unary {
                        expr: Box::new(expr),
                        operator,
                    })
                } else {
                    let expr = parse_expression(&node.children[1])?;
                    let operator = &node.children[0].children[0].token.clone().unwrap().str;
                    Ok(NodeExpression::Update {
                        child: Box::new(expr),
                        operator,
                        is_prefix: true,
                    })
                }
            }
        }
        "PostfixExpression" | "PostfixExpressionNotBrace" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                Ok(NodeExpression::Update {
                    operator: &node.children[1].children[0].token.clone().unwrap().str,
                    is_prefix: false,
                    child: Box::new(left),
                })
            }
        }
        "SelectorExpression" | "SelectorExpressionNotBrace" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                let left = parse_expression(&node.children[0])?;
                Ok(NodeExpression::Selector {
                    child: Box::new(left),
                    selector: parse_selector(&node.children[1])?,
                })
            }
        }
        "SliceExpression" => parse_slice_expression(node),
        "ThrowExpression" => Ok(NodeExpression::Throw {
            expr: Box::new(parse_expression(&node.children[1])?),
        }),
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
