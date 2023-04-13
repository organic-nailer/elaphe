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
        statements: Vec<Box<NodeStatement<'input>>>,
    },
    IfStatement {
        condition: Box<NodeExpression<'input>>,
        if_true_stmt: Box<NodeStatement<'input>>,
        if_false_stmt: Option<Box<NodeStatement<'input>>>,
    },
    RethrowStatement,
    TryFinallyStatement {
        block_try: Box<NodeStatement<'input>>,
        block_finally: Box<NodeStatement<'input>>,
    },
    TryOnStatement {
        block_try: Box<NodeStatement<'input>>,
        on_part_list: Vec<TryOnPart<'input>>,
    },
    ForStatement {
        init: Option<Box<NodeStatement<'input>>>,
        condition: Option<Box<NodeExpression<'input>>>,
        update: Option<Vec<Box<NodeExpression<'input>>>>,
        stmt: Box<NodeStatement<'input>>,
    },
    WhileStatement {
        condition: Box<NodeExpression<'input>>,
        stmt: Box<NodeStatement<'input>>,
    },
    DoStatement {
        condition: Box<NodeExpression<'input>>,
        stmt: Box<NodeStatement<'input>>,
    },
    ReturnStatement {
        value: Option<Box<NodeExpression<'input>>>,
    },
    SwitchStatement {
        expr: Box<NodeExpression<'input>>,
        case_list: Vec<SwitchCase<'input>>,
        default_case: Option<DefaultCase<'input>>,
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

pub struct SwitchCase<'input> {
    pub label_list: Vec<Box<NodeExpression<'input>>>,
    pub expr: Box<NodeExpression<'input>>,
    pub stmt_list: Vec<Box<NodeStatement<'input>>>,
}

pub struct DefaultCase<'input> {
    pub label_list: Vec<Box<NodeExpression<'input>>>,
    pub stmt_list: Vec<Box<NodeStatement<'input>>>,
}

pub struct TryOnPart<'input> {
    pub catch_part: Option<TryCatchPart<'input>>,
    pub exc_type: Option<DartType<'input>>,
    pub block: Box<NodeStatement<'input>>,
}

pub struct TryCatchPart<'input> {
    pub id_error: Identifier<'input>,
    pub id_trace: Option<Identifier<'input>>,
}
