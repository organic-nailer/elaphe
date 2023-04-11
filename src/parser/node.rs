pub enum NodeExpression<'input> {
    Binary {
        left: Box<NodeExpression<'input>>,
        operator: &'input str,
        right: Box<NodeExpression<'input>>,
    },
    NumericLiteral {
        value: &'input str,
    },
}
