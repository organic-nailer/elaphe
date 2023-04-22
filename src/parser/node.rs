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
    Update {
        operator: &'input str,
        is_prefix: bool,
        child: Box<NodeExpression<'input>>,
    },
    Assignment {
        operator: &'input str,
        left: Box<NodeExpression<'input>>,
        right: Box<NodeExpression<'input>>,
    },
    TypeTest {
        child: Box<NodeExpression<'input>>,
        type_test: TypeTest<'input>,
    },
    TypeCast {
        child: Box<NodeExpression<'input>>,
        type_cast: DartType<'input>,
    },
    NumericLiteral {
        value: &'input str,
    },
    StringLiteral {
        str_list: Vec<StringWithInterpolation<'input>>,
    },
    BooleanLiteral {
        value: &'input str,
    },
    NullLiteral,
    ListLiteral {
        element_list: Vec<CollectionElement<'input>>,
    },
    SetOrMapLiteral {
        element_list: Vec<CollectionElement<'input>>,
    },
    Identifier {
        identifier: Identifier<'input>,
    },
    Selector {
        child: Box<NodeExpression<'input>>,
        selector: Selector<'input>,
    },
    Slice {
        start: Option<Box<NodeExpression<'input>>>,
        end: Option<Box<NodeExpression<'input>>>,
        step: Option<Box<NodeExpression<'input>>>,
    },
    Throw {
        expr: Box<NodeExpression<'input>>,
    },
    This,
}

pub enum NodeStatement<'input> {
    Labeled {
        label: Identifier<'input>,
        stmt: Box<NodeStatement<'input>>,
    },
    Break {
        label: Option<Identifier<'input>>,
    },
    Continue {
        label: Option<Identifier<'input>>,
    },
    Return {
        value: Option<Box<NodeExpression<'input>>>,
    },
    Empty,
    Expression {
        expr: Box<NodeExpression<'input>>,
    },
    Block {
        statements: Vec<Box<NodeStatement<'input>>>,
    },
    Rethrow,
    VariableDeclarationList {
        decl_list: Vec<VariableDeclaration<'input>>,
    },
    FunctionDeclaration {
        signature: FunctionSignature<'input>,
        body: Box<NodeStatement<'input>>,
    },
    ClassDeclaration {
        identifier: Identifier<'input>,
        member_list: Vec<Member<'input>>,
    },
    If {
        condition: Box<NodeExpression<'input>>,
        if_true_stmt: Box<NodeStatement<'input>>,
        if_false_stmt: Option<Box<NodeStatement<'input>>>,
    },
    TryFinally {
        block_try: Box<NodeStatement<'input>>,
        block_finally: Box<NodeStatement<'input>>,
    },
    TryOn {
        block_try: Box<NodeStatement<'input>>,
        on_part_list: Vec<TryOnPart<'input>>,
    },
    For {
        init: Option<Box<NodeStatement<'input>>>,
        condition: Option<Box<NodeExpression<'input>>>,
        update: Option<Vec<Box<NodeExpression<'input>>>>,
        stmt: Box<NodeStatement<'input>>,
    },
    While {
        condition: Box<NodeExpression<'input>>,
        stmt: Box<NodeStatement<'input>>,
    },
    Do {
        condition: Box<NodeExpression<'input>>,
        stmt: Box<NodeStatement<'input>>,
    },
    Switch {
        expr: Box<NodeExpression<'input>>,
        case_list: Vec<SwitchCase<'input>>,
        default_case: Option<DefaultCase<'input>>,
    },
}

pub enum Selector<'input> {
    Index {
        expr: Box<NodeExpression<'input>>,
    },
    Attr {
        identifier: Identifier<'input>,
    },
    Method {
        identifier: Identifier<'input>,
        arguments: Vec<CallParameter<'input>>,
    },
    Args {
        args: Vec<CallParameter<'input>>,
    },
}

pub struct CallParameter<'input> {
    pub identifier: Option<Identifier<'input>>,
    pub expr: Box<NodeExpression<'input>>,
}

pub enum IdentifierKind {
    Normal,
    BuiltIn,
    Other,
}

pub struct Identifier<'input> {
    pub value: &'input str,
    pub kind: IdentifierKind,
}

pub struct LibraryDeclaration<'input> {
    pub import_list: Vec<LibraryImport<'input>>,
    pub top_level_declaration_list: Vec<Box<NodeStatement<'input>>>,
}

pub struct LibraryImport<'input> {
    pub uri: &'input str,
    pub identifier: Option<Identifier<'input>>,
    pub combinator_list: Vec<Combinator<'input>>,
}

pub struct Combinator<'input> {
    pub is_show: bool,
    pub target_list: Vec<Identifier<'input>>,
}

pub struct FunctionSignature<'input> {
    pub return_type: Option<DartType<'input>>,
    pub name: Identifier<'input>,
    pub param: FunctionParamSignature<'input>,
}

pub struct FunctionParamSignature<'input> {
    pub normal_list: Vec<FunctionParameter<'input>>,
    pub option_list: Vec<FunctionParameter<'input>>,
    pub named_list: Vec<FunctionParameter<'input>>,
}

pub struct FunctionParameter<'input> {
    pub identifier: Identifier<'input>,
    pub expr: Option<Box<NodeExpression<'input>>>,
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

pub enum CollectionElement<'input> {
    ExpressionElement {
        expr: Box<NodeExpression<'input>>,
    },
    MapElement {
        key_expr: Box<NodeExpression<'input>>,
        value_expr: Box<NodeExpression<'input>>,
    },
}

pub struct ConstructorSignature<'input> {
    pub name: Option<Identifier<'input>>,
    pub param: FunctionParamSignature<'input>,
}

pub enum Member<'input> {
    MethodImpl {
        signature: FunctionSignature<'input>,
        body: Box<NodeStatement<'input>>,
    },
    ConstructorImpl {
        signature: ConstructorSignature<'input>,
        body: Box<NodeStatement<'input>>,
    },
    VariableDecl {
        decl_list: Vec<VariableDeclaration<'input>>,
    },
}

pub struct TypeTest<'input> {
    pub dart_type: DartType<'input>,
    pub check_matching: bool,
}

pub struct StringWithInterpolation<'input> {
    pub string_list: Vec<&'input str>,
    pub interpolation_list: Vec<NodeExpression<'input>>,
}
