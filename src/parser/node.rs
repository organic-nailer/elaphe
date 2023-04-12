pub enum NodeExpression<'input> {
    Binary {
        left: Box<NodeExpression<'input>>,
        operator: &'input str,
        right: Box<NodeExpression<'input>>,
    },
    BooleanLiteral {
        value: &'input str,
    },
    NumericLiteral {
        value: &'input str,
    },
    StringLiteral {
        str_list: Vec<&'input str>,
    },
    Identifier {
        identifier: Identifier<'input>,
    },
    Selector {
        left: Box<NodeExpression<'input>>,
        operator: Selector<'input>,
    },
}

pub enum NodeStatement<'input> {
    FunctionDeclaration {
        signature: FunctionSignature<'input>,
        body: Box<NodeStatement<'input>>,
    },
    ExpressionStatement {
        expr: Box<NodeExpression<'input>>,
    },
    BlockStatement {
        statements: Vec<NodeStatement<'input>>,
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

pub struct LibraryDeclaration<'input> {
    pub top_level_declaration_list: Vec<Box<NodeStatement<'input>>>,
}
pub struct FunctionSignature<'input> {
    pub name: Identifier<'input>,
    pub param: Vec<Identifier<'input>>,
}
