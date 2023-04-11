use std::{fmt, error::Error};
use dart_parser_generator::{parser_generator::{TransitionMap, TransitionKind}, grammar::EPSILON};

use crate::tokenizer::{Token, TokenKind};

use self::node::NodeExpression;

pub mod node;

pub fn parse<'input>(input: Vec<Token<'input>>, transition_map: TransitionMap) -> Result<NodeExpression, Box<dyn Error>> {
    let internal_node = parse_internally(input, transition_map);

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
        println!("stack: {:?}, index: {}", stack, parse_index);
        let transition = transition_map.transitions.get(&(
            stack.last().unwrap().0.clone(), 
            input[parse_index].kind_str()
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
                    input[parse_index].kind_str()));
                node_stack.push(NodeInternal::Leaf {
                    rule_name: input[parse_index].kind_str(),
                    value: input[parse_index].clone(),
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
                let new_node = NodeInternal::Parent {
                    rule_name: rule.left.to_string(),
                    children,
                };

                let next_transition = transition_map.transitions.get(&(stack.last().unwrap().0.clone(), rule.left.to_string()));
                if next_transition.is_none() {
                    println!("Shift-Reduce Conflict: {:?}", rule);
                    break;
                }
                let next_transition = next_transition.unwrap();
                stack.push((next_transition.target.clone().unwrap(), rule.left.to_string()));

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

pub enum NodeInternal<'input> {
    Parent {
        rule_name: String,
        children: Vec<NodeInternal<'input>>,
    },
    Leaf {
        rule_name: String,
        value: Token<'input>,
    },
}

fn parse_expression<'input>(node: &NodeInternal<'input>) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    match node {
        NodeInternal::Parent { rule_name, children } => {
            match rule_name.as_str() {
                "AdditiveExpression" |
                "MultiplicativeExpression" => {
                    if children.len() == 1 {
                        parse_expression(&children[0])
                    } else {
                        let left = parse_expression(&children[0])?;
                        let right = parse_expression(&children[2])?;
                        Ok(NodeExpression::Binary {
                            left: Box::new(left),
                            right: Box::new(right),
                            operator: get_value_unsafe(&children[1]).value,
                        })
                    }
                }
                "PrimaryExpression" => {
                    if children.len() == 1 {
                        parse_expression(&children[0])
                    } else {
                        parse_expression(&children[1])
                    }
                }
                v => {
                    Err(format!("Parse Error: {} is not valid rule", v).into())
                }
            }
        }
        NodeInternal::Leaf { rule_name, value } => {
            match rule_name.as_str() {
                "Number" => {
                    Ok(NodeExpression::NumericLiteral { 
                        value: value.value
                    })
                }
                v => {
                    Err(format!("Parse Error: {} is not valid rule", v).into())
                }
            }
        }
    }
}

fn get_value_unsafe<'input>(node: &NodeInternal<'input>) -> Token<'input> {
    match node {
        NodeInternal::Parent { .. } => panic!("NodeInternal::Parent is not Leaf"),
        NodeInternal::Leaf { value, .. } => value.clone(),
    }
}
