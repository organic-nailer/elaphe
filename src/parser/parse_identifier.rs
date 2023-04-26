use anyhow::{bail, Result};

use super::{
    node::{Identifier, IdentifierKind},
    node_internal::NodeInternal,
};

pub fn parse_identifier_list<'input>(
    node: &NodeInternal<'input>,
) -> Result<Vec<Identifier<'input>>> {
    if node.rule_name == "IdentifierList" {
        if node.children.len() == 1 {
            return Ok(vec![parse_identifier(&node.children[0])?]);
        } else {
            let mut list = parse_identifier_list(&node.children[0])?;
            list.push(parse_identifier(&node.children[2])?);
            return Ok(list);
        }
    }

    bail!("Parse Error in parse_identifier_list: {}", node.rule_name);
}

pub fn parse_identifier<'input>(node: &NodeInternal<'input>) -> Result<Identifier<'input>> {
    if node.rule_name == "Identifier" || node.rule_name == "TypeIdentifier" {
        let child_node = &node.children[0];
        if child_node.rule_name == "IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.token.clone().unwrap().str,
                kind: IdentifierKind::Normal,
            });
        } else if child_node.rule_name == "BUILT_IN_IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.children[0].token.clone().unwrap().str,
                kind: IdentifierKind::BuiltIn,
            });
        } else if child_node.rule_name == "OTHER_IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.children[0].token.clone().unwrap().str,
                kind: IdentifierKind::Other,
            });
        }
    }

    bail!("Parse Error in parse_identifier: {}", node.rule_name);
}
