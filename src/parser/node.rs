pub enum NodeExpression<'input> {
    Binary {
        left: Box<NodeExpression<'input>>,
        operator: &'input str,
        right: Box<NodeExpression<'input>>,
    },
    Conditional {
        condition: Box<NodeExpression<'input>>,
        true_expr: Box<NodeExpression<'input>>,
        false_expr: Box<NodeExpression<'input>>,
    },
    Unary {
        operator: &'input str,
        expr: Box<NodeExpression<'input>>,
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
    VariableDeclarationList {
        decl_list: Vec<VariableDeclaration<'input>>,
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
    pub return_type: Option<DartType<'input>>,
    pub name: Identifier<'input>,
    pub param: Vec<Identifier<'input>>,
}

pub enum DartType<'input> {
    Named {
        type_name: DartTypeName<'input>,
        type_arguments: Vec<DartType<'input>>,
        is_nullable: bool,
    },
    Void,
}

pub struct DartTypeName<'input> {
    pub identifier: Identifier<'input>,
    pub module: Option<Identifier<'input>>,
}

pub struct VariableDeclaration<'input> {
    pub identifier: Identifier<'input>,
    pub expr: Option<Box<NodeExpression<'input>>>,
}
