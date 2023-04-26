use anyhow::{bail, Result};

use crate::tokenizer::{BUILT_IN_IDENTIFIER, OTHER_IDENTIFIER};

use super::{
    node::{
        CollectionElement, Identifier, IdentifierKind, NodeExpression, StringWithInterpolation,
    },
    node_internal::NodeInternal,
    parse_expression::parse_expression,
    util::flatten,
};

pub fn parse_string_literal_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<StringWithInterpolation<'input>>> {
    if node.rule_name == "StringLiteralList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_string_literal(&node.children[0])?]);
        } else {
            return flatten(
                parse_string_literal_list(&node.children[0]),
                parse_string_literal(&node.children[1])?,
            );
        }
    }

    bail!(
        "Parse Error in parse_string_literal_list: {}",
        node.rule_name
    );
}

fn parse_string_literal<'input>(
    node: &NodeInternal<'input>,
) -> Result<StringWithInterpolation<'input>> {
    if node.rule_name == "StringLiteral" {
        if node.children.len() == 1 {
            return Ok(parse_string_no_brace(&node.children[0])?);
        } else {
            let mut left = parse_string_no_brace(&node.children[0])?;
            let center = parse_string_interpolation(&node.children[1])?;
            let right = parse_string_no_brace(&node.children[2])?;
            left.string_list.extend(center.string_list);
            left.string_list.extend(right.string_list);
            left.interpolation_list.extend(center.interpolation_list);
            left.interpolation_list.extend(right.interpolation_list);
            return Ok(left);
        }
    }

    bail!("Parse Error in parse_string_literal: {}", node.rule_name);
}

fn parse_string_no_brace<'input>(
    node: &NodeInternal<'input>,
) -> Result<StringWithInterpolation<'input>> {
    let regex_non_escaped_dollar = regex::Regex::new(r#"(^|[^\\])(\\\\)*\$"#).unwrap();
    let regex_identifier_or_keyword = regex::Regex::new(r"^[a-zA-Z_\$][0-9a-zA-Z_\$]*").unwrap();
    if node.rule_name == "STRING_BEGIN_END"
        || node.rule_name == "STRING_BEGIN_MID"
        || node.rule_name == "STRING_MID_MID"
        || node.rule_name == "STRING_MID_END"
    {
        let text = node.token.clone().unwrap().str;
        let mut id_start_end_list: Vec<(usize, usize)> = vec![];
        for m in regex_non_escaped_dollar.find_iter(&text) {
            let end = m.end();
            match regex_identifier_or_keyword.find(&text[end..]) {
                Some(m) => {
                    id_start_end_list.push((end, end + m.end()));
                }
                None => {
                    bail!("$ must be followed by identifier: {}", text);
                }
            }
        }

        let mut string_list: Vec<&'input str> = vec![];
        let mut interpolation_list: Vec<NodeExpression<'input>> = vec![];

        let mut start = 0;
        for (id_start, id_end) in id_start_end_list {
            string_list.push(&text[start..id_start - 1]);
            if &text[id_start..id_end] == "this" {
                interpolation_list.push(NodeExpression::This);
            } else if BUILT_IN_IDENTIFIER.contains(&&text[id_start..id_end]) {
                interpolation_list.push(NodeExpression::Identifier {
                    identifier: Identifier {
                        value: &text[id_start..id_end],
                        kind: IdentifierKind::BuiltIn,
                    },
                });
            } else if OTHER_IDENTIFIER.contains(&&text[id_start..id_end]) {
                interpolation_list.push(NodeExpression::Identifier {
                    identifier: Identifier {
                        value: &text[id_start..id_end],
                        kind: IdentifierKind::Other,
                    },
                });
            } else {
                interpolation_list.push(NodeExpression::Identifier {
                    identifier: Identifier {
                        value: &text[id_start..id_end],
                        kind: IdentifierKind::Normal,
                    },
                });
            }
            start = id_end;
        }

        string_list.push(&text[start..]);

        return Ok(StringWithInterpolation {
            string_list,
            interpolation_list,
        });
    }

    bail!("Parse Error in parse_string_no_brace: {}", node.rule_name);
}

fn parse_string_interpolation<'input>(
    node: &NodeInternal<'input>,
) -> Result<StringWithInterpolation<'input>> {
    if node.rule_name == "StringInterpolation" {
        if node.children.len() == 1 {
            return Ok(StringWithInterpolation {
                string_list: vec![],
                interpolation_list: vec![parse_expression(&node.children[0])?],
            });
        } else {
            let mut left = parse_string_interpolation(&node.children[0])?;
            left.string_list
                .push(&node.children[1].token.clone().unwrap().str);
            left.interpolation_list
                .push(parse_expression(&node.children[2])?);
            return Ok(left);
        }
    }

    bail!(
        "Parse Error in parse_string_interpolation: {}",
        node.rule_name
    );
}

pub fn parse_list_literal<'input>(node: &NodeInternal<'input>) -> Result<NodeExpression<'input>> {
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

    bail!("Parse Error in parse_list_literal: {}", node.rule_name);
}

pub fn parse_set_or_map_literal<'input>(
    node: &NodeInternal<'input>,
) -> Result<NodeExpression<'input>> {
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

    bail!(
        "Parse Error in parse_set_or_map_literal: {}",
        node.rule_name
    );
}

fn parse_element_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<CollectionElement<'input>>> {
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

    bail!("Parse Error in parse_element_list: {}", node.rule_name);
}

fn parse_element<'input>(node: &NodeInternal<'input>) -> Result<CollectionElement<'input>> {
    if node.rule_name == "Element" {
        if node.children[0].rule_name == "ExpressionElement" {
            return parse_expression_element(&node.children[0]);
        } else if node.children[0].rule_name == "MapElement" {
            return parse_map_element(&node.children[0]);
        }
    }

    bail!("Parse Error in parse_element: {}", node.rule_name);
}

fn parse_expression_element<'input>(
    node: &NodeInternal<'input>,
) -> Result<CollectionElement<'input>> {
    if node.rule_name == "ExpressionElement" {
        return Ok(CollectionElement::ExpressionElement {
            expr: Box::new(parse_expression(&node.children[0])?),
        });
    }

    bail!(
        "Parse Error in parse_expression_element: {}",
        node.rule_name
    );
}

fn parse_map_element<'input>(node: &NodeInternal<'input>) -> Result<CollectionElement<'input>> {
    if node.rule_name == "MapElement" {
        return Ok(CollectionElement::MapElement {
            key_expr: Box::new(parse_expression(&node.children[0])?),
            value_expr: Box::new(parse_expression(&node.children[2])?),
        });
    }

    bail!("Parse Error in parse_map_element: {}", node.rule_name);
}
