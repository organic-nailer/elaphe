use crate::tokenizer::Token;

pub struct NodeInternal<'input> {
    pub rule_name: String,
    pub children: Vec<NodeInternal<'input>>,
    pub token: Option<Token<'input>>,
}
