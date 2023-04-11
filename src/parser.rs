use dart_parser_generator::{
    grammar::EPSILON,
    parser_generator::{TransitionKind, TransitionMap},
};
use std::{error::Error, fmt};

use crate::tokenizer::{Token, TokenKind};

use self::node::{CallParameter, Identifier, NodeExpression, Selector};

pub mod node;

pub fn parse<'input>(
    input: Vec<Token<'input>>,
    transition_map: TransitionMap,
) -> Result<NodeExpression, Box<dyn Error>> {
    let internal_node = parse_internally(input, transition_map);
    println!("accepted from the parser");

    parse_expression(&internal_node)
}

fn parse_internally(input: Vec<Token>, transition_map: TransitionMap) -> NodeInternal {
    let mut stack: Vec<(String, String)> = Vec::new();
    let mut node_stack: Vec<NodeInternal> = Vec::new();
    let mut parse_index = 0;
    let mut accepted = false;

    // Stackの先頭のTokenは使わないのでなんでもよい
    stack.push((String::from("I0"), "".to_string()));
    while parse_index < input.len() || !stack.is_empty() {
        // println!("stack: {:?}, index: {}", stack, parse_index);
        let transition = transition_map.transitions.get(&(
            stack.last().unwrap().0.clone(),
            input[parse_index].kind_str(),
        ));

        if transition.is_none() {
            println!("No Transition Error: {:?}", input[parse_index]);
            break;
        }
        let transition = transition.unwrap();
        match transition.kind {
            TransitionKind::Shift => {
                stack.push((
                    transition.target.clone().unwrap(),
                    input[parse_index].kind_str(),
                ));
                node_stack.push(NodeInternal {
                    rule_name: input[parse_index].kind_str(),
                    token: Some(input[parse_index].clone()),
                    children: Vec::new(),
                });
                parse_index += 1;
            }
            TransitionKind::Reduce => {
                let rule = transition.rule.clone().unwrap();
                let mut children = Vec::new();
                let child_size = if rule.right.len() == 1 && rule.right[0] == EPSILON {
                    0
                } else {
                    rule.right.len()
                };
                for _ in 0..child_size {
                    stack.pop();
                    children.push(node_stack.pop().unwrap());
                }
                children.reverse();
                let new_node = NodeInternal {
                    rule_name: rule.left.to_string(),
                    token: None,
                    children,
                };

                let next_transition = transition_map
                    .transitions
                    .get(&(stack.last().unwrap().0.clone(), rule.left.to_string()));
                if next_transition.is_none() {
                    println!("Shift-Reduce Conflict: {:?}", rule);
                    break;
                }
                let next_transition = next_transition.unwrap();
                stack.push((
                    next_transition.target.clone().unwrap(),
                    rule.left.to_string(),
                ));

                node_stack.push(new_node);
            }
            TransitionKind::Accept => {
                accepted = true;
                break;
            }
        }
    }

    if accepted {
        node_stack.pop().unwrap()
    } else {
        panic!("Parse Error");
    }
}

pub struct NodeInternal<'input> {
    rule_name: String,
    children: Vec<NodeInternal<'input>>,
    token: Option<Token<'input>>,
}

fn parse_expression<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    match node.rule_name.as_str() {
        "AdditiveExpression" | "MultiplicativeExpression" => {
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
        "PrimaryExpression" => {
            if node.children.len() == 1 {
                parse_expression(&node.children[0])
            } else {
                parse_expression(&node.children[1])
            }
        }
        "Number" => Ok(NodeExpression::NumericLiteral {
            value: node.token.clone().unwrap().str,
        }),
        "Identifier" => Ok(NodeExpression::Identifier {
            identifier: Identifier {
                value: node.token.clone().unwrap().str,
            },
        }),
        v => Err(format!("Parse Error: {} is not valid rule in expression", v).into()),
    }
}

fn flatten<T>(left: Result<Vec<T>, Box<dyn Error>>, right: T) -> Result<Vec<T>, Box<dyn Error>> {
    let mut flt = left?;
    flt.push(right);
    Ok(flt)
}

fn parse_selector<'input>(node: &NodeInternal<'input>) -> Result<Selector<'input>, Box<dyn Error>> {
    if node.rule_name != "Selector" {
        return Err("Parse Error in selector".into());
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
        v => Err(format!("Parse Error: {} is not valid rule in selector", v).into()),
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

    Err("Parse Error in normal_argument".into())
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

    Err("Parse Error argument_list".into())
}
