use std::error::Error;

use super::{
    node::{Identifier, IdentifierKind},
    node_internal::NodeInternal,
    util::gen_error,
};

pub fn parse_identifier<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "Identifier" {
        let child_node = &node.children[0];
        if child_node.rule_name == "IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.token.clone().unwrap().str,
                kind: IdentifierKind::Normal,
            });
        } else if child_node.rule_name == "BUILT_IN_IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.token.clone().unwrap().str,
                kind: IdentifierKind::BuiltIn,
            });
        } else if child_node.rule_name == "OTHER_IDENTIFIER" {
            return Ok(Identifier {
                value: child_node.token.clone().unwrap().str,
                kind: IdentifierKind::Other,
            });
        }
    }

    Err(gen_error("parse_identifier", &node.rule_name))
}
