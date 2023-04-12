use std::error::Error;

use super::{node::Identifier, node_internal::NodeInternal, util::gen_error};

pub fn parse_identifier<'input>(
    node: &NodeInternal<'input>,
) -> Result<Identifier<'input>, Box<dyn Error>> {
    if node.rule_name == "Identifier" {
        return Ok(Identifier {
            value: node.token.clone().unwrap().str,
        });
    }

    Err(gen_error("parse_identifier", &node.rule_name))
}
