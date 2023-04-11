pub enum NodeExpression<'input> {
    Binary {
        left: Box<NodeExpression<'input>>,
        operator: &'input str,
        right: Box<NodeExpression<'input>>,
    },
    NumericLiteral {
        value: &'input str,
    },
    Identifier {
        identifier: Identifier<'input>,
    },
    Selector {
        left: Box<NodeExpression<'input>>,
        operator: Selector<'input>,
    },
}

pub enum Selector<'input> {
    Args { args: Vec<CallParameter<'input>> },
}

pub struct CallParameter<'input> {
    pub identifier: Option<Identifier<'input>>,
    pub expr: Box<NodeExpression<'input>>,
}

pub struct Identifier<'input> {
    pub value: &'input str,
}
