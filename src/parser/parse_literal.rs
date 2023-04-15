use std::error::Error;

use super::{
    node::{CollectionElement, NodeExpression},
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    util::{flatten, gen_error},
};

pub fn parse_string_literal_list<'input>(
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

pub fn parse_list_literal<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    if node.rule_name == "ListLiteral" {
        if node.children.len() == 2 || node.children.len() == 3 {
            return Ok(NodeExpression::ListLiteral {
                element_list: vec![],
            });
        } else if node.children.len() == 4 {
            if node.children[0].token.clone().unwrap().str == "[" {
                return Ok(NodeExpression::ListLiteral {
                    element_list: parse_element_list(&node.children[1])?,
                });
            } else {
                return Ok(NodeExpression::ListLiteral {
                    element_list: vec![],
                });
            }
        } else if node.children.len() == 5 {
            return Ok(NodeExpression::ListLiteral {
                element_list: parse_element_list(&node.children[2])?,
            });
        } else {
            return Ok(NodeExpression::ListLiteral {
                element_list: parse_element_list(&node.children[3])?,
            });
        }
    }

    Err(gen_error("parse_list_literal", &node.rule_name))
}

pub fn parse_set_or_map_literal<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>, Box<dyn Error>> {
    if node.rule_name == "SetOrMapLiteral" {
        if node.children.len() == 2 || node.children.len() == 3 {
            return Ok(NodeExpression::SetOrMapLiteral {
                element_list: vec![],
            });
        } else if node.children.len() == 4 {
            if node.children[0].token.clone().unwrap().str == "{" {
                return Ok(NodeExpression::SetOrMapLiteral {
                    element_list: parse_element_list(&node.children[1])?,
                });
            } else {
                return Ok(NodeExpression::SetOrMapLiteral {
                    element_list: vec![],
                });
            }
        } else if node.children.len() == 5 {
            return Ok(NodeExpression::SetOrMapLiteral {
                element_list: parse_element_list(&node.children[2])?,
            });
        } else {
            return Ok(NodeExpression::SetOrMapLiteral {
                element_list: parse_element_list(&node.children[3])?,
            });
        }
    }

    Err(gen_error("parse_set_or_map_literal", &node.rule_name))
}

fn parse_element_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<CollectionElement<'input>>, Box<dyn Error>> {
    if node.rule_name == "ElementList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_element(&node.children[0])?]);
        } else {
            return flatten(
                parse_element_list(&node.children[0]),
                parse_element(&node.children[2])?,
            );
        }
    }

    Err(gen_error("parse_element_list", &node.rule_name))
}

fn parse_element<'input>(
    node: &NodeInternal<'input>,
) -> Result<CollectionElement<'input>, Box<dyn Error>> {
    if node.rule_name == "Element" {
        if node.children[0].rule_name == "ExpressionElement" {
            return parse_expression_element(&node.children[0]);
        } else if node.children[0].rule_name == "MapElement" {
            return parse_map_element(&node.children[0]);
        }
    }

    Err(gen_error("parse_element", &node.rule_name))
}

fn parse_expression_element<'input>(
    node: &NodeInternal<'input>,
) -> Result<CollectionElement<'input>, Box<dyn Error>> {
    if node.rule_name == "ExpressionElement" {
        return Ok(CollectionElement::ExpressionElement {
            expr: Box::new(parse_expression(&node.children[0])?),
        });
    }

    Err(gen_error("parse_expression_element", &node.rule_name))
}

fn parse_map_element<'input>(
    node: &NodeInternal<'input>,
) -> Result<CollectionElement<'input>, Box<dyn Error>> {
    if node.rule_name == "MapElement" {
        return Ok(CollectionElement::MapElement {
            key_expr: Box::new(parse_expression(&node.children[0])?),
            value_expr: Box::new(parse_expression(&node.children[2])?),
        });
    }

    Err(gen_error("parse_map_element", &node.rule_name))
}
