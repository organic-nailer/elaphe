use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::bytecode::ByteCode;
use crate::executioncontext::{BlockContext, ExecutionContext, VariableScope};
use crate::parser::node::{DartType, Identifier, NodeExpression, NodeStatement, Selector};
use crate::{bytecode::OpCode, pyobject::PyObject};

// use self::runclass::run_class;
use self::runfunction::run_function;

pub mod runclass;
pub mod runfunction;
pub mod runroot;

struct DefaultScope {
    break_label: u32,
    continue_label: Option<u32>,
}

pub struct ByteCompiler<'ctx, 'value> {
    pub byte_operations: RefCell<Vec<OpCode>>,
    context_stack: Vec<Rc<RefCell<dyn ExecutionContext + 'ctx>>>,
    jump_label_table: RefCell<HashMap<u32, u8>>,
    jump_label_key_index: RefCell<u32>,
    default_scope_stack: Vec<DefaultScope>,
    break_label_table: HashMap<String, u32>,
    continue_label_table: HashMap<String, u32>,
    source: &'value str,
}

impl<'ctx, 'value> ByteCompiler<'ctx, 'value> {
    // fn compile_import(&mut self, node: &LibraryImport) {
    //     let uri = &node.uri;
    //     let len = uri.len();
    //     let uri = &uri[1..len - 1];

    //     let identifier = match &node.identifier {
    //         Some(v) => Some(v.value),
    //         None => None,
    //     };

    //     // uri形式
    //     // import A.B as C
    //     // → import "elaphe/A/B.d.dart" as C;
    //     // from ..A.B import C as D
    //     // → import "../A/B/C.d.dart" as D;
    //     if uri.contains(":") {
    //         panic!("invalid import uri: {}", uri);
    //     }

    //     if uri.starts_with(".") {
    //         // 相対パスの場合
    //         let mut path_splitted: Vec<&str> = uri.split("/").collect();

    //         // ドットの数を積む
    //         let dot_len = path_splitted[0].len();
    //         self.push_load_const(PyObject::Int(dot_len as i32, false));
    //         path_splitted.remove(0);

    //         // 最後尾のモジュールをタプルで積む
    //         let import_mod = path_splitted.pop().unwrap();
    //         let import_mod_p = self.push_load_const(PyObject::SmallTuple {
    //             children: vec![PyObject::new_string(import_mod.to_string(), false)],
    //             add_ref: false,
    //         });

    //         // 名前でインポート
    //         let p = (**self.context_stack.last().unwrap())
    //             .borrow_mut()
    //             .register_or_get_name(&path_splitted.join("."));
    //         self.push_op(OpCode::ImportName(p));

    //         // インポート先のモジュール
    //         self.push_op(OpCode::ImportFrom(import_mod_p));

    //         // 格納先
    //         let store_name = match identifier {
    //             Some(v) => v,
    //             None => import_mod,
    //         };
    //         let store_name_p = (**self.context_stack.last().unwrap())
    //             .borrow_mut()
    //             .declare_variable(&store_name.to_string());
    //         self.push_op(OpCode::StoreName(store_name_p));
    //         self.push_op(OpCode::PopTop);
    //     } else {
    //         // 絶対パスの場合はelaphe/から始まり、最後尾は.d.dartで終わる必要がある
    //         if !uri.starts_with("elaphe/") {
    //             panic!("invalid import uri: {}", uri);
    //         }
    //         if !uri.ends_with(".d.dart") {
    //             // .d.dartで終わらないものは無視する
    //             return;
    //         }
    //         let path_splitted: Vec<&str> = uri[7..uri.len() - 7].split("/").collect();

    //         // 0を積む
    //         self.push_load_const(PyObject::Int(0, false));

    //         // Noneを積む
    //         self.push_load_const(PyObject::None(false));

    //         // 名前でインポート
    //         let import_name = path_splitted.join(".");
    //         let import_name_p = (**self.context_stack.last().unwrap())
    //             .borrow_mut()
    //             .register_or_get_name(&import_name);
    //         self.push_op(OpCode::ImportName(import_name_p));

    //         match identifier {
    //             None => {
    //                 // import A.B
    //                 let store_name = path_splitted[0].to_string();
    //                 let store_name_p = (**self.context_stack.last().unwrap())
    //                     .borrow_mut()
    //                     .declare_variable(&store_name);
    //                 self.push_op(OpCode::StoreName(store_name_p));
    //             }
    //             Some(v) => {
    //                 if path_splitted.len() == 1 {
    //                     // import A as B
    //                     let p = (**self.context_stack.last().unwrap())
    //                         .borrow_mut()
    //                         .declare_variable(&v.to_string());
    //                     self.push_op(OpCode::StoreName(p));
    //                 } else {
    //                     // import A.B.C as D
    //                     let second_name = path_splitted[1].to_string();
    //                     let p = (**self.context_stack.last().unwrap())
    //                         .borrow_mut()
    //                         .register_or_get_name(&second_name);
    //                     self.push_op(OpCode::ImportFrom(p));
    //                     for i in 2..path_splitted.len() {
    //                         self.push_op(OpCode::RotTwo);
    //                         self.push_op(OpCode::PopTop);
    //                         let p = (**self.context_stack.last().unwrap())
    //                             .borrow_mut()
    //                             .register_or_get_name(&path_splitted[i].to_string());
    //                         self.push_op(OpCode::ImportFrom(p));
    //                     }
    //                     let p = (**self.context_stack.last().unwrap())
    //                         .borrow_mut()
    //                         .declare_variable(&v.to_string());
    //                     self.push_op(OpCode::StoreName(p));
    //                     self.push_op(OpCode::PopTop);
    //                 }
    //             }
    //         }
    //     }
    // }

    fn compile_stmt(&mut self, node: &'value NodeStatement, label: Option<&String>) {
        match node {
            NodeStatement::FunctionDeclaration { signature, body } => {
                self.compile_declare_function(
                    &signature.name.value.to_string(),
                    &signature.param,
                    body,
                    None,
                    None,
                    |_| (),
                );
            }
            NodeStatement::VariableDeclarationList { decl_list } => {
                for declaration in decl_list {
                    match &declaration.expr {
                        Some(e) => {
                            self.compile_expr(e, None);
                            let value = declaration.identifier.value.to_string();
                            let position = (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&value);
                            if self.context_stack.last().unwrap().borrow().is_global() {
                                // トップレベル変数の場合
                                self.push_op(OpCode::StoreName(position));
                            } else {
                                // ローカル変数の場合
                                let local_position = self
                                    .context_stack
                                    .last()
                                    .unwrap()
                                    .borrow()
                                    .get_local_variable(&value);
                                self.push_op(OpCode::StoreFast(local_position));
                            }
                        }
                        None => {
                            let value = declaration.identifier.value.to_string();
                            (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&value);
                        }
                    }
                }
            }
            NodeStatement::ExpressionStatement { expr } => {
                self.compile_expr(expr, None);
                self.push_op(OpCode::PopTop);
            }
            NodeStatement::BlockStatement { statements } => {
                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                for child in statements {
                    self.compile_stmt(child, None);
                }
                self.context_stack.pop();
            }
            NodeStatement::IfStatement {
                condition,
                if_true_stmt,
                if_false_stmt,
            } => {
                match if_false_stmt {
                    Some(if_false_stmt) => {
                        // if expr stmt else stmt
                        self.compile_expr(condition, None);
                        let label_false_starts = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_false_starts));

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile_stmt(if_true_stmt, None);
                        self.context_stack.pop();

                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::JumpAbsolute(label_if_ends));

                        self.set_jump_label_value(label_false_starts);

                        self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                            outer: self.context_stack.last().unwrap().clone(),
                            variables: vec![],
                        })));
                        self.compile_stmt(if_false_stmt, None);
                        self.context_stack.pop();

                        self.set_jump_label_value(label_if_ends);
                    }
                    None => {
                        // if expr stmt
                        self.compile_expr(condition, None);
                        let label_if_ends = self.gen_jump_label();
                        self.push_op(OpCode::PopJumpIfFalse(label_if_ends));
                        self.compile_stmt(if_true_stmt, None);

                        self.set_jump_label_value(label_if_ends);
                    }
                }
            }
            NodeStatement::RethrowStatement => {
                self.push_op(OpCode::Reraise);
            }
            NodeStatement::TryFinallyStatement {
                block_try,
                block_finally,
            } => {
                let label_finally = self.gen_jump_label();
                let label_end = self.gen_jump_label();

                self.push_op(OpCode::SetupFinally(label_finally));
                let label_finally_zero = self.byte_operations.borrow().len() as u8;

                // 通常フロー
                self.compile_stmt(block_try, None);
                self.push_op(OpCode::PopBlock);
                self.compile_stmt(block_finally, None);
                self.push_op(OpCode::JumpAbsolute(label_end));

                // 例外が起きたときのフロー
                self.set_jump_label_value_offset(label_finally, label_finally_zero);
                self.compile_stmt(block_finally, None);
                self.push_op(OpCode::Reraise);

                self.set_jump_label_value(label_end);
            }
            NodeStatement::TryOnStatement {
                block_try,
                on_part_list,
            } => {
                let label_finally = self.gen_jump_label();
                let label_end = self.gen_jump_label();

                self.push_op(OpCode::SetupFinally(label_finally));
                let label_finally_zero = self.byte_operations.borrow().len() as u8;

                // 通常のフロー
                self.compile_stmt(block_try, None);
                self.push_op(OpCode::PopBlock);
                self.push_op(OpCode::JumpAbsolute(label_end));

                // 例外時のフロー
                self.set_jump_label_value_offset(label_finally, label_finally_zero);
                for on_part in on_part_list {
                    let label_next = self.gen_jump_label();

                    self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                        outer: self.context_stack.last().unwrap().clone(),
                        variables: vec![],
                    })));

                    // catchする型の指定がある場合はloadして検証する
                    match &on_part.exc_type {
                        Some(v) => {
                            if let DartType::Named {
                                type_name,
                                type_arguments: _,
                                is_nullable: _,
                            } = v
                            {
                                let name = type_name.identifier.value.to_string();
                                let p = (**self.context_stack.last().unwrap())
                                    .borrow_mut()
                                    .register_or_get_name(&name);
                                self.push_op(OpCode::LoadName(p));
                                self.push_op(OpCode::JumpIfNotExcMatch(label_next));
                            }
                        }
                        None => (),
                    }

                    // スタックの状態:
                    // [trace_back, value, exception] -> TOP
                    match &on_part.catch_part {
                        Some(catch_part) => {
                            // on E catch() { }
                            // self.push_op(OpCode::PopTop);
                            // 実行してみると最初にpopしたものがexception、
                            // 2番目がtraceback objectになっている
                            // なぜ？

                            let name = catch_part.id_error.value.to_string();
                            (**self.context_stack.last().unwrap())
                                .borrow_mut()
                                .declare_variable(&name);
                            self.push_store_var(&name);

                            match &catch_part.id_trace {
                                Some(id_trace) => {
                                    let name = id_trace.value.to_string();
                                    (**self.context_stack.last().unwrap())
                                        .borrow_mut()
                                        .declare_variable(&name);
                                    self.push_store_var(&name);
                                }
                                None => {
                                    self.push_op(OpCode::PopTop);
                                }
                            }

                            self.push_op(OpCode::PopTop);
                        }
                        None => {
                            // on E { }
                            self.push_op(OpCode::PopTop);
                            self.push_op(OpCode::PopTop);
                            self.push_op(OpCode::PopTop);
                        }
                    }

                    self.compile_stmt(&on_part.block, None);
                    self.push_op(OpCode::PopExcept);
                    self.push_op(OpCode::JumpAbsolute(label_end));

                    self.set_jump_label_value(label_next);

                    self.context_stack.pop();
                }

                self.push_op(OpCode::Reraise);

                self.set_jump_label_value(label_end);
            }
            NodeStatement::ForStatement {
                init,
                condition,
                update,
                stmt,
            } => {
                // init is statement
                // condition is expression
                // update is expression list
                let label_for_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_for_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_loop_start);
                }
                if let Some(node) = init {
                    self.compile_stmt(node, None);
                }
                self.set_jump_label_value(label_loop_start);
                if let Some(node) = condition {
                    self.compile_expr(node, None);
                    self.push_op(OpCode::PopJumpIfFalse(label_for_end));
                }

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None);
                self.context_stack.pop();

                if let Some(node_list) = update {
                    for node in node_list {
                        self.compile_expr(node, None);
                        self.push_op(OpCode::PopTop);
                    }
                }
                self.push_op(OpCode::JumpAbsolute(label_loop_start));
                self.set_jump_label_value(label_for_end);
                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::WhileStatement { condition, stmt } => {
                let label_while_end = self.gen_jump_label();
                let label_loop_start = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_while_end,
                    continue_label: Some(label_loop_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_loop_start);
                }
                self.set_jump_label_value(label_loop_start);
                self.compile_expr(condition, None);
                self.push_op(OpCode::PopJumpIfFalse(label_while_end));

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None);
                self.context_stack.pop();

                self.push_op(OpCode::JumpAbsolute(label_loop_start));

                self.set_jump_label_value(label_while_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::DoStatement { condition, stmt } => {
                let label_do_start = self.gen_jump_label();
                let label_do_end = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_do_end,
                    continue_label: Some(label_do_start),
                });
                if let Some(stmt_label) = label {
                    self.continue_label_table
                        .insert(stmt_label.to_string(), label_do_start);
                }
                self.set_jump_label_value(label_do_start);

                self.context_stack.push(Rc::new(RefCell::new(BlockContext {
                    outer: self.context_stack.last().unwrap().clone(),
                    variables: vec![],
                })));
                self.compile_stmt(stmt, None);
                self.context_stack.pop();

                self.compile_expr(condition, None);
                self.push_op(OpCode::PopJumpIfTrue(label_do_start));
                self.set_jump_label_value(label_do_end);

                self.default_scope_stack.pop();
                if let Some(stmt_label) = label {
                    self.continue_label_table.remove(stmt_label);
                }
            }
            NodeStatement::ReturnStatement { value } => {
                match value {
                    Some(v) => self.compile_expr(v, None),
                    None => {
                        self.push_load_const(PyObject::None(false));
                    }
                }
                self.push_op(OpCode::ReturnValue);
            }
            NodeStatement::SwitchStatement {
                expr,
                case_list,
                default_case,
            } => {
                self.compile_expr(expr, None);
                let label_switch_end = self.gen_jump_label();
                self.default_scope_stack.push(DefaultScope {
                    break_label: label_switch_end,
                    continue_label: match self.default_scope_stack.last() {
                        Some(v) => v.continue_label,
                        None => None,
                    },
                });
                let case_labels: Vec<u32> =
                    case_list.iter().map(|_| self.gen_jump_label()).collect();
                for case_index in 0..case_list.len() {
                    let case = &case_list[case_index];
                    let case_label = case_labels[case_index];
                    self.push_op(OpCode::DupTop);
                    self.compile_expr(&case.expr, None);
                    self.push_op(OpCode::compare_op_from_str("=="));
                    self.push_op(OpCode::PopJumpIfTrue(case_label));
                }
                let label_default_start = self.gen_jump_label();
                if let Some(_) = default_case {
                    self.push_op(OpCode::JumpAbsolute(label_default_start));
                }
                for case_index in 0..case_list.len() {
                    let case = &case_list[case_index];
                    let case_label = case_labels[case_index];
                    self.set_jump_label_value(case_label);
                    for stmt in &case.stmt_list {
                        self.compile_stmt(stmt, None);
                    }
                }
                if let Some(default_case) = default_case {
                    self.set_jump_label_value(label_default_start);
                    for stmt in &default_case.stmt_list {
                        self.compile_stmt(stmt, None);
                    }
                }
                self.set_jump_label_value(label_switch_end);
                self.default_scope_stack.pop();
            }
        }
    }

    fn compile_expr(&mut self, node: &'value NodeExpression, label: Option<&String>) {
        match node {
            NodeExpression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expr(left, None);
                self.compile_expr(right, None);
                match *operator {
                    "==" | "!=" | ">=" | ">" | "<=" | "<" => {
                        self.push_op(OpCode::compare_op_from_str(operator))
                    }
                    "<<" => self.push_op(OpCode::BinaryLShift),
                    ">>" => self.push_op(OpCode::BinaryRShift),
                    "&" => self.push_op(OpCode::BinaryAnd),
                    "^" => self.push_op(OpCode::BinaryXor),
                    "|" => self.push_op(OpCode::BinaryOr),
                    "+" => self.push_op(OpCode::BinaryAdd),
                    "-" => self.push_op(OpCode::BinarySubtract),
                    "*" => self.push_op(OpCode::BinaryMultiply),
                    "/" => self.push_op(OpCode::BinaryTrueDivide),
                    "%" => self.push_op(OpCode::BinaryModulo),
                    "~/" => self.push_op(OpCode::BinaryFloorDivide),
                    _ => panic!("unknown operator: {}", *operator),
                }
            }
            NodeExpression::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let label_conditional_end = self.gen_jump_label();
                let label_false_start = self.gen_jump_label();
                self.compile_expr(condition, None);
                self.push_op(OpCode::PopJumpIfFalse(label_false_start));
                self.compile_expr(true_expr, None);
                self.push_op(OpCode::JumpAbsolute(label_conditional_end));
                self.set_jump_label_value(label_false_start);
                self.compile_expr(false_expr, None);
                self.set_jump_label_value(label_conditional_end);
            }
            NodeExpression::Unary { operator, expr } => {
                self.compile_expr(expr, None);
                match *operator {
                    "-" => self.push_op(OpCode::UnaryNegative),
                    "!" => self.push_op(OpCode::UnaryNot),
                    "~" => self.push_op(OpCode::UnaryInvert),
                    _ => panic!("unknown unary operator: {}", *operator),
                }
            }
            NodeExpression::BooleanLiteral { value } => {
                self.push_load_const(PyObject::new_boolean(value, false));
            }
            NodeExpression::NumericLiteral { value } => {
                self.push_load_const(PyObject::new_numeric(value, false));
            }
            NodeExpression::StringLiteral { str_list } => {
                let value = str_list
                    .iter()
                    .map(|v| {
                        let len = v.len();
                        &v[1..len - 1]
                    })
                    .collect::<Vec<&'value str>>()
                    .join("");

                self.push_load_const(PyObject::new_string(value.to_string(), false));
            }
            NodeExpression::Identifier { identifier } => {
                let value = identifier.value.to_string();
                self.push_load_var(&value);
            }
            NodeExpression::Selector { left, operator } => {
                // 右辺値として処理される場合
                self.compile_expr(left, None);

                match operator {
                    Selector::Args { args } => {
                        let mut name_list: Vec<&str> = vec![];
                        for param in args {
                            self.compile_expr(&param.expr, None);
                            if let Some(v) = &param.identifier {
                                name_list.push(v.value);
                            }
                        }
                        if !name_list.is_empty() {
                            self.push_load_const(PyObject::SmallTuple {
                                children: name_list
                                    .iter()
                                    .map(|v| PyObject::new_string(v.to_string(), false))
                                    .collect(),
                                add_ref: false,
                            });
                            self.push_op(OpCode::CallFunctionKw(args.len() as u8));
                        } else {
                            self.push_op(OpCode::CallFunction(args.len() as u8));
                        }
                    }
                }
            }
        }
        // match node {
        //     Node::BinaryExpression {
        //         operator,
        //         left,
        //         right,
        //     } => {
        //         if *operator == "??" {
        //             self.compile(left, None);
        //             self.push_op(OpCode::DupTop);
        //             self.push_load_const(PyObject::None(false));
        //             self.push_op(OpCode::compare_op_from_str("=="));
        //             let label_end = self.gen_jump_label();
        //             self.push_op(OpCode::PopJumpIfFalse(label_end));
        //             self.push_op(OpCode::PopTop);
        //             self.compile(right, None);
        //             self.set_jump_label_value(label_end);
        //         } else if *operator == "||" {
        //             self.compile(left, None);
        //             self.push_op(OpCode::DupTop);
        //             let label_end = self.gen_jump_label();
        //             self.push_op(OpCode::PopJumpIfTrue(label_end));
        //             self.push_op(OpCode::PopTop);
        //             self.compile(right, None);
        //             self.set_jump_label_value(label_end);
        //         } else if *operator == "&&" {
        //             self.compile(left, None);
        //             self.push_op(OpCode::DupTop);
        //             let label_end = self.gen_jump_label();
        //             self.push_op(OpCode::PopJumpIfFalse(label_end));
        //             self.push_op(OpCode::PopTop);
        //             self.compile(right, None);
        //             self.set_jump_label_value(label_end);
        //         } else {
        //             self.compile(left, None);
        //             self.compile(right, None);
        //             match *operator {
        //                 "==" | "!=" | ">=" | ">" | "<=" | "<" => {
        //                     self.push_op(OpCode::compare_op_from_str(operator))
        //                 }
        //                 "<<" => self.push_op(OpCode::BinaryLShift),
        //                 ">>" => self.push_op(OpCode::BinaryRShift),
        //                 "&" => self.push_op(OpCode::BinaryAnd),
        //                 "^" => self.push_op(OpCode::BinaryXor),
        //                 "|" => self.push_op(OpCode::BinaryOr),
        //                 "+" => self.push_op(OpCode::BinaryAdd),
        //                 "-" => self.push_op(OpCode::BinarySubtract),
        //                 "*" => self.push_op(OpCode::BinaryMultiply),
        //                 "/" => self.push_op(OpCode::BinaryTrueDivide),
        //                 "%" => self.push_op(OpCode::BinaryModulo),
        //                 "~/" => self.push_op(OpCode::BinaryFloorDivide),
        //                 _ => panic!("unknown operator: {}", *operator),
        //             }
        //         }
        //     }
        //     Node::ConditionalExpression {
        //         condition,
        //         if_true_expr,
        //         if_false_expr,
        //     } => {
        //         let label_conditional_end = self.gen_jump_label();
        //         let label_false_start = self.gen_jump_label();
        //         self.compile(condition, None);
        //         self.push_op(OpCode::PopJumpIfFalse(label_false_start));
        //         self.compile(if_true_expr, None);
        //         self.push_op(OpCode::JumpAbsolute(label_conditional_end));
        //         self.set_jump_label_value(label_false_start);
        //         self.compile(if_false_expr, None);
        //         self.set_jump_label_value(label_conditional_end);
        //     }
        //     Node::UnaryOpExpression { operator, child } => {
        //         self.compile(child, None);
        //         match *operator {
        //             "-" => self.push_op(OpCode::UnaryNegative),
        //             "!" => self.push_op(OpCode::UnaryNot),
        //             "~" => self.push_op(OpCode::UnaryInvert),
        //             _ => panic!("unknown unary operator: {}", *operator),
        //         }
        //     }
        //     Node::UpdateExpression {
        //         operator,
        //         is_prefix,
        //         child,
        //     } => {
        //         if let Node::IdentifierNode { identifier } = &**child {
        //             let value = identifier.value.to_string();
        //             if *is_prefix {
        //                 // 前置
        //                 self.push_load_var(&value);
        //                 self.push_load_const(PyObject::Int(1, false));
        //                 match *operator {
        //                     "++" => self.push_op(OpCode::InplaceAdd),
        //                     "--" => self.push_op(OpCode::InplaceSubtract),
        //                     _ => (),
        //                 }
        //                 self.push_op(OpCode::DupTop);
        //                 self.push_store_var(&value);
        //             } else {
        //                 // 後置
        //                 self.push_load_var(&value);
        //                 self.push_op(OpCode::DupTop);
        //                 self.push_load_const(PyObject::Int(1, false));
        //                 match *operator {
        //                     "++" => self.push_op(OpCode::InplaceAdd),
        //                     "--" => self.push_op(OpCode::InplaceSubtract),
        //                     _ => (),
        //                 }
        //                 self.push_store_var(&value);
        //             }
        //         } else {
        //             panic!("Invalid AST. Increment target must be an identifier.");
        //         }
        //     }
        //     Node::TypeTestExpression { child, type_test } => {
        //         // isinstance(child, type_test.)
        //         self.push_load_var(&"isinstance".to_string());
        //         self.compile(child, None);
        //         if let DartType::Named {
        //             type_name,
        //             type_arguments: _,
        //             is_nullable: _,
        //         } = &type_test.dart_type
        //         {
        //             let name = type_name.identifier.value;
        //             let p = (**self.context_stack.last().unwrap())
        //                 .borrow_mut()
        //                 .register_or_get_name(&name.to_string());
        //             self.push_op(OpCode::LoadName(p));
        //         } else {
        //             panic!("Invalid Test Expression");
        //         }
        //         self.push_op(OpCode::CallFunction(2));
        //         if !type_test.check_matching {
        //             self.push_op(OpCode::UnaryNot);
        //         }
        //     }
        //     Node::TypeCastExpression {
        //         child,
        //         type_cast: _,
        //     } => {
        //         // 実行時には型がないので無視
        //         self.compile(child, None);
        //     }
        //     Node::AssignmentExpression {
        //         operator,
        //         left,
        //         right,
        //     } => {
        //         match *operator {
        //             "=" => {
        //                 self.compile(right, None);
        //                 // DartではAssignment Expressionが代入先の最終的な値を残す
        //                 self.push_op(OpCode::DupTop);

        //                 match &**left {
        //                     Node::IdentifierNode { identifier } => {
        //                         let value = identifier.value.to_string();
        //                         self.push_store_var(&value);
        //                     }
        //                     Node::SelectorExpression { child, selector } => {
        //                         self.compile(child, None);

        //                         match selector {
        //                             Selector::Args { args: _ } => panic!("Invalid lhs value."),
        //                             Selector::Method {
        //                                 identifier: _,
        //                                 arguments: _,
        //                             } => panic!("Invalid lhs value."),
        //                             Selector::Attr { identifier } => {
        //                                 let name = identifier.value;
        //                                 let p = (**self.context_stack.last().unwrap())
        //                                     .borrow_mut()
        //                                     .register_or_get_name(&name.to_string());
        //                                 self.push_op(OpCode::StoreAttr(p));
        //                             }
        //                             Selector::Index { expr } => {
        //                                 self.compile(expr, None);
        //                                 self.push_op(OpCode::StoreSubScr);
        //                             }
        //                         }
        //                     }
        //                     _ => panic!("Invalid lhs value."),
        //                 }
        //             }
        //             "*=" | "/=" | "~/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^="
        //             | "|=" => match &**left {
        //                 Node::IdentifierNode { identifier } => {
        //                     let value = identifier.value.to_string();
        //                     self.push_load_var(&value);

        //                     self.compile(right, None);
        //                     match *operator {
        //                         "*=" => self.push_op(OpCode::InplaceMultiply),
        //                         "/=" => self.push_op(OpCode::InplaceTrueDivide),
        //                         "~/=" => self.push_op(OpCode::InplaceFloorDivide),
        //                         "%=" => self.push_op(OpCode::InplaceModulo),
        //                         "+=" => self.push_op(OpCode::InplaceAdd),
        //                         "-=" => self.push_op(OpCode::InplaceSubtract),
        //                         "<<=" => self.push_op(OpCode::InplaceLShift),
        //                         ">>=" => self.push_op(OpCode::InplaceRShift),
        //                         "&=" => self.push_op(OpCode::InplaceAnd),
        //                         "^=" => self.push_op(OpCode::InplaceXor),
        //                         "|=" => self.push_op(OpCode::InplaceOr),
        //                         _ => (),
        //                     }
        //                     self.push_op(OpCode::DupTop);

        //                     self.push_store_var(&value);
        //                 }
        //                 Node::SelectorExpression { child, selector } => {
        //                     self.compile(child, None);
        //                     match selector {
        //                         Selector::Args { args: _ } => panic!("Invalid lhs value."),
        //                         Selector::Method {
        //                             identifier: _,
        //                             arguments: _,
        //                         } => panic!("Invalid lhs value."),
        //                         Selector::Attr { identifier } => {
        //                             let name = identifier.value;
        //                             let p = (**self.context_stack.last().unwrap())
        //                                 .borrow_mut()
        //                                 .register_or_get_name(&name.to_string());

        //                             self.push_op(OpCode::DupTop);
        //                             self.push_op(OpCode::LoadAttr(p));

        //                             self.compile(right, None);
        //                             match *operator {
        //                                 "*=" => self.push_op(OpCode::InplaceMultiply),
        //                                 "/=" => self.push_op(OpCode::InplaceTrueDivide),
        //                                 "~/=" => self.push_op(OpCode::InplaceFloorDivide),
        //                                 "%=" => self.push_op(OpCode::InplaceModulo),
        //                                 "+=" => self.push_op(OpCode::InplaceAdd),
        //                                 "-=" => self.push_op(OpCode::InplaceSubtract),
        //                                 "<<=" => self.push_op(OpCode::InplaceLShift),
        //                                 ">>=" => self.push_op(OpCode::InplaceRShift),
        //                                 "&=" => self.push_op(OpCode::InplaceAnd),
        //                                 "^=" => self.push_op(OpCode::InplaceXor),
        //                                 "|=" => self.push_op(OpCode::InplaceOr),
        //                                 _ => (),
        //                             }
        //                             self.push_op(OpCode::DupTop);

        //                             self.push_op(OpCode::RotThree);
        //                             self.push_op(OpCode::StoreAttr(p));
        //                         }
        //                         Selector::Index { expr } => {
        //                             self.compile(expr, None);
        //                             self.push_op(OpCode::DupTopTwo);
        //                             self.push_op(OpCode::BinarySubScr);

        //                             self.compile(right, None);
        //                             match *operator {
        //                                 "*=" => self.push_op(OpCode::InplaceMultiply),
        //                                 "/=" => self.push_op(OpCode::InplaceTrueDivide),
        //                                 "~/=" => self.push_op(OpCode::InplaceFloorDivide),
        //                                 "%=" => self.push_op(OpCode::InplaceModulo),
        //                                 "+=" => self.push_op(OpCode::InplaceAdd),
        //                                 "-=" => self.push_op(OpCode::InplaceSubtract),
        //                                 "<<=" => self.push_op(OpCode::InplaceLShift),
        //                                 ">>=" => self.push_op(OpCode::InplaceRShift),
        //                                 "&=" => self.push_op(OpCode::InplaceAnd),
        //                                 "^=" => self.push_op(OpCode::InplaceXor),
        //                                 "|=" => self.push_op(OpCode::InplaceOr),
        //                                 _ => (),
        //                             }
        //                             self.push_op(OpCode::DupTop);
        //                             self.push_op(OpCode::RotFour);

        //                             self.push_op(OpCode::RotFour);
        //                             self.push_op(OpCode::StoreSubScr);
        //                         }
        //                     }
        //                 }
        //                 _ => panic!("Invalid lhs value."),
        //             },
        //             "??=" => match &**left {
        //                 Node::IdentifierNode { identifier } => {
        //                     let value = identifier.value.to_string();

        //                     self.push_load_var(&value);
        //                     self.push_op(OpCode::DupTop);
        //                     self.push_load_const(PyObject::None(false));
        //                     self.push_op(OpCode::compare_op_from_str("=="));
        //                     let label_end = self.gen_jump_label();
        //                     self.push_op(OpCode::PopJumpIfFalse(label_end));

        //                     self.push_op(OpCode::PopTop);
        //                     self.compile(right, None);
        //                     self.push_op(OpCode::DupTop);
        //                     self.push_store_var(&value);
        //                     self.set_jump_label_value(label_end);
        //                 }
        //                 Node::SelectorExpression { child, selector } => {
        //                     self.compile(child, None);
        //                     match selector {
        //                         Selector::Args { args: _ } => panic!("Invalid lhs value."),
        //                         Selector::Method {
        //                             identifier: _,
        //                             arguments: _,
        //                         } => panic!("Invalid lhs value."),
        //                         Selector::Attr { identifier } => {
        //                             let name = identifier.value;
        //                             let p = (**self.context_stack.last().unwrap())
        //                                 .borrow_mut()
        //                                 .register_or_get_name(&name.to_string());

        //                             self.push_op(OpCode::DupTop);
        //                             self.push_op(OpCode::LoadAttr(p));

        //                             self.push_op(OpCode::DupTop);
        //                             self.push_load_const(PyObject::None(false));
        //                             self.push_op(OpCode::compare_op_from_str("=="));
        //                             let label_false = self.gen_jump_label();
        //                             self.push_op(OpCode::PopJumpIfFalse(label_false));

        //                             self.push_op(OpCode::PopTop);
        //                             self.compile(right, None);
        //                             self.push_op(OpCode::DupTop);
        //                             self.push_op(OpCode::RotThree);
        //                             self.push_op(OpCode::RotThree);
        //                             self.push_op(OpCode::StoreAttr(p));
        //                             let label_end = self.gen_jump_label();
        //                             self.push_op(OpCode::JumpAbsolute(label_end));

        //                             self.set_jump_label_value(label_false);
        //                             self.push_op(OpCode::RotTwo);
        //                             self.push_op(OpCode::PopTop);

        //                             self.set_jump_label_value(label_end);
        //                         }
        //                         Selector::Index { expr } => {
        //                             self.compile(expr, None);
        //                             self.push_op(OpCode::DupTopTwo);
        //                             self.push_op(OpCode::BinarySubScr);

        //                             self.push_op(OpCode::DupTop);
        //                             self.push_load_const(PyObject::None(false));
        //                             self.push_op(OpCode::compare_op_from_str("=="));
        //                             let label_false = self.gen_jump_label();
        //                             self.push_op(OpCode::PopJumpIfFalse(label_false));

        //                             self.push_op(OpCode::PopTop);
        //                             self.compile(right, None);
        //                             self.push_op(OpCode::DupTop);
        //                             self.push_op(OpCode::RotFour);
        //                             self.push_op(OpCode::RotFour);
        //                             self.push_op(OpCode::StoreSubScr);
        //                             let label_end = self.gen_jump_label();
        //                             self.push_op(OpCode::JumpAbsolute(label_end));

        //                             self.set_jump_label_value(label_false);
        //                             self.push_op(OpCode::RotThree);
        //                             self.push_op(OpCode::PopTop);
        //                             self.push_op(OpCode::PopTop);

        //                             self.set_jump_label_value(label_end);
        //                         }
        //                     }
        //                 }
        //                 _ => panic!("Invalid lhs value."),
        //             },
        //             _ => panic!("Unknown assignment operator: {}", operator),
        //         }
        //     }
        //     Node::NumericLiteral { value } => {
        //         self.push_load_const(PyObject::new_numeric(value, false));
        //     }
        //     Node::StringLiteral { literal_list } => {
        //         let value = literal_list
        //             .iter()
        //             .map(|v| {
        //                 let len = v.len();
        //                 &v[1..len - 1]
        //             })
        //             .collect::<Vec<&'value str>>()
        //             .join("");

        //         self.push_load_const(PyObject::new_string(value.to_string(), false));
        //     }
        //     Node::BooleanLiteral { value } => {
        //         self.push_load_const(PyObject::new_boolean(value, false));
        //     }
        //     Node::NullLiteral => {
        //         self.push_load_const(PyObject::None(false));
        //     }
        //     Node::ListLiteral { element_list } => {
        //         let size = element_list.len() as u8;
        //         for elem in element_list {
        //             match elem {
        //                 CollectionElement::ExpressionElement { expr } => {
        //                     self.compile(expr, None);
        //                 }
        //                 CollectionElement::MapElement {
        //                     key_expr: _,
        //                     value_expr: _,
        //                 } => {
        //                     panic!("Invalid List Literal");
        //                 }
        //             }
        //         }
        //         self.push_op(OpCode::BuildList(size));
        //     }
        //     Node::SetOrMapLiteral { element_list } => {
        //         let first_elem = element_list.first();
        //         let is_map = if let Some(elem) = first_elem {
        //             match elem {
        //                 CollectionElement::ExpressionElement { expr: _ } => false,
        //                 CollectionElement::MapElement {
        //                     key_expr: _,
        //                     value_expr: _,
        //                 } => true,
        //             }
        //         } else {
        //             true
        //         };

        //         if is_map {
        //             let size = element_list.len() as u8;
        //             for elem in element_list {
        //                 match elem {
        //                     CollectionElement::ExpressionElement { expr: _ } => {
        //                         panic!("Invalid Map Literal");
        //                     }
        //                     CollectionElement::MapElement {
        //                         key_expr,
        //                         value_expr,
        //                     } => {
        //                         self.compile(key_expr, None);
        //                         self.compile(value_expr, None);
        //                     }
        //                 }
        //             }
        //             self.push_op(OpCode::BuildMap(size));
        //         } else {
        //             let size = element_list.len() as u8;
        //             for elem in element_list {
        //                 match elem {
        //                     CollectionElement::ExpressionElement { expr } => {
        //                         self.compile(expr, None);
        //                     }
        //                     CollectionElement::MapElement {
        //                         key_expr: _,
        //                         value_expr: _,
        //                     } => {
        //                         panic!("Invalid Set Literal");
        //                     }
        //                 }
        //             }
        //             self.push_op(OpCode::BuildSet(size));
        //         }
        //     }
        //     Node::IdentifierNode { identifier } => {
        //         let value = identifier.value.to_string();
        //         self.push_load_var(&value);
        //     }
        //     Node::Arguments { children } => {
        //         let mut name_list: Vec<&str> = vec![];
        //         for param in children {
        //             self.compile(&param.expr, None);
        //             if let Some(v) = &param.identifier {
        //                 name_list.push(v.value);
        //             }
        //         }
        //         if !name_list.is_empty() {
        //             self.push_load_const(PyObject::SmallTuple {
        //                 children: name_list
        //                     .iter()
        //                     .map(|v| PyObject::new_string(v.to_string(), false))
        //                     .collect(),
        //                 add_ref: false,
        //             });
        //             self.push_op(OpCode::CallFunctionKw(children.len() as u8));
        //         } else {
        //             self.push_op(OpCode::CallFunction(children.len() as u8));
        //         }
        //     }
        //     Node::SelectorExpression { child, selector } => {
        //         // 右辺値として処理される場合
        //         self.compile(child, None);

        //         match selector {
        //             Selector::Args { args } => {
        //                 // Node::Argumentsがどうにかしてくれる
        //                 self.compile(args, None);
        //             }
        //             Selector::Attr { identifier } => {
        //                 let name = identifier.value;
        //                 let p = (**self.context_stack.last().unwrap())
        //                     .borrow_mut()
        //                     .register_or_get_name(&name.to_string());
        //                 self.push_op(OpCode::LoadAttr(p));
        //             }
        //             Selector::Index { expr } => {
        //                 self.compile(expr, None);

        //                 self.push_op(OpCode::BinarySubScr);
        //             }
        //             Selector::Method {
        //                 identifier,
        //                 arguments,
        //             } => {
        //                 let name = identifier.value;
        //                 let p = (**self.context_stack.last().unwrap())
        //                     .borrow_mut()
        //                     .register_or_get_name(&name.to_string());
        //                 self.push_op(OpCode::LoadMethod(p));

        //                 if let Node::Arguments { children } = &**arguments {
        //                     let mut name_list: Vec<&str> = vec![];
        //                     for param in children {
        //                         self.compile(&param.expr, None);
        //                         if let Some(v) = &param.identifier {
        //                             name_list.push(&v.value);
        //                         }
        //                     }
        //                     if !name_list.is_empty() {
        //                         self.push_load_const(PyObject::SmallTuple {
        //                             children: name_list
        //                                 .iter()
        //                                 .map(|v| PyObject::new_string(v.to_string(), false))
        //                                 .collect(),
        //                             add_ref: false,
        //                         });
        //                         self.push_op(OpCode::CallFunctionKw(children.len() as u8));
        //                     } else {
        //                         self.push_op(OpCode::CallMethod(children.len() as u8))
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     Node::SliceExpression { start, end, step } => {
        //         match start {
        //             Some(v) => {
        //                 self.compile(v, None);
        //             }
        //             None => {
        //                 self.push_load_const(PyObject::None(false));
        //             }
        //         }

        //         match end {
        //             Some(v) => {
        //                 self.compile(v, None);
        //             }
        //             None => {
        //                 self.push_load_const(PyObject::None(false));
        //             }
        //         }

        //         match step {
        //             Some(v) => {
        //                 self.compile(v, None);
        //                 self.push_op(OpCode::BuildSlice(3));
        //             }
        //             None => {
        //                 self.push_op(OpCode::BuildSlice(2));
        //             }
        //         }
        //     }
        //     Node::ThrowExpression { expr } => {
        //         self.compile(expr, None);
        //         self.push_op(OpCode::RaiseVarargs(1));
        //     }
        //     Node::ThisExpression => {
        //         let p = self
        //             .context_stack
        //             .last()
        //             .unwrap()
        //             .borrow()
        //             .get_local_variable(&"self".to_string());
        //         self.push_op(OpCode::LoadFast(p));
        //     }

        //     Node::LabeledStatement { label, stmt } => {
        //         let label_str = label.value.to_string();
        //         let label_id = self.gen_jump_label();

        //         // break用のラベルはこの時点で用意する
        //         self.break_label_table.insert(label_str.clone(), label_id);

        //         self.compile(stmt, Some(&label_str));

        //         self.set_jump_label_value(label_id);
        //         self.break_label_table.remove(&label_str);
        //     }
        //     Node::BreakStatement { label } => match label {
        //         Some(identifier) => {
        //             let label_str = identifier.value;
        //             match self.break_label_table.get(label_str) {
        //                 Some(v) => {
        //                     self.push_op(OpCode::JumpAbsolute(*v));
        //                 }
        //                 None => panic!("label {} is not existing in this scope.", label_str),
        //             }
        //         }
        //         None => match self.default_scope_stack.last() {
        //             Some(v) => {
        //                 self.push_op(OpCode::JumpAbsolute(v.break_label));
        //             }
        //             None => panic!("break statement is not available here"),
        //         },
        //     },
        //     Node::ContinueStatement { label } => match label {
        //         Some(identifier) => {
        //             let label_str = identifier.value;
        //             match self.continue_label_table.get(label_str) {
        //                 Some(v) => {
        //                     self.push_op(OpCode::JumpAbsolute(*v));
        //                 }
        //                 None => panic!("label {} is not existing in this scope.", label_str),
        //             }
        //         }
        //         None => match self.default_scope_stack.last() {
        //             Some(v) => match v.continue_label {
        //                 Some(continue_label) => self.push_op(OpCode::JumpAbsolute(continue_label)),
        //                 None => panic!("continue statement is not available here"),
        //             },
        //             None => panic!("continue statement is not available here"),
        //         },
        //     },
        //     Node::EmptyStatement => {}

        //     Node::VariableDeclarationList { decl_list } => {
        //         for declaration in decl_list {
        //             match &declaration.expr {
        //                 Some(e) => {
        //                     self.compile(e, None);
        //                     let value = declaration.identifier.value.to_string();
        //                     let position = (**self.context_stack.last().unwrap())
        //                         .borrow_mut()
        //                         .declare_variable(&value);
        //                     if self.context_stack.last().unwrap().borrow().is_global() {
        //                         // トップレベル変数の場合
        //                         self.push_op(OpCode::StoreName(position));
        //                     } else {
        //                         // ローカル変数の場合
        //                         let local_position = self
        //                             .context_stack
        //                             .last()
        //                             .unwrap()
        //                             .borrow()
        //                             .get_local_variable(&value);
        //                         self.push_op(OpCode::StoreFast(local_position));
        //                     }
        //                 }
        //                 None => {
        //                     let value = declaration.identifier.value.to_string();
        //                     (**self.context_stack.last().unwrap())
        //                         .borrow_mut()
        //                         .declare_variable(&value);
        //                 }
        //             }
        //         }
        //     }
        //     Node::FunctionDeclaration { signature, body } => {
        //         self.comiple_declare_function(
        //             &signature.name.value.to_string(),
        //             &signature.param,
        //             body,
        //             None,
        //             None,
        //             |_| (),
        //         );
        //     }
        //     Node::ClassDeclaration {
        //         identifier,
        //         member_list,
        //     } => {
        //         self.push_op(OpCode::LoadBuildClass);

        //         let name = identifier.value.to_string();
        //         self.push_load_const(run_class(
        //             &"main.py".to_string(),
        //             &name,
        //             member_list,
        //             self,
        //             self.source,
        //         ));

        //         self.push_load_const(PyObject::new_string(name.clone(), false));

        //         self.push_op(OpCode::MakeFunction(0));

        //         self.push_load_const(PyObject::new_string(name.clone(), false));

        //         self.push_op(OpCode::CallFunction(2));

        //         let p = (**self.context_stack.last().unwrap())
        //             .borrow_mut()
        //             .declare_variable(&name);
        //         self.push_op(OpCode::StoreName(p));
        //     }
        // }
    }

    fn compile_declare_function<F: FnOnce(&mut ByteCompiler<'ctx, 'value>)>(
        &mut self,
        name: &String,
        param: &Vec<Identifier<'value>>,
        // param: &'value FunctionParamSignature,
        body: &'value Box<NodeStatement>,
        function_name_prefix: Option<String>,
        implicit_arg: Option<&String>,
        preface: F,
    ) {
        let mut argument_list: Vec<String> = vec![];
        if let Some(name) = implicit_arg {
            argument_list.push(name.clone());
        }
        for p in param {
            let name = p.value.to_string();
            argument_list.push(name);
        }
        let num_normal_args = param.len() as u32;
        let num_kw_only_args = 0;
        // for p in &param.normal_list {
        //     let name = p.identifier.value.to_string();
        //     argument_list.push(name);
        // }
        // for p in &param.option_list {
        //     let name = p.identifier.value.to_string();
        //     argument_list.push(name);
        // }
        // for p in &param.named_list {
        //     let name = p.identifier.value.to_string();
        //     argument_list.push(name);
        // }

        // let mut num_normal_args = param.normal_list.len() as u32 + param.option_list.len() as u32;
        // if implicit_arg.is_some() {
        //     num_normal_args += 1;
        // }
        // let num_kw_only_args = param.named_list.len() as u32;
        let py_code = run_function(
            &"main.py".to_string(),
            &name,
            argument_list,
            num_normal_args,
            0,
            num_kw_only_args,
            self,
            body,
            self.source,
            preface,
        );

        // // 通常引数のデフォルト値の設定
        // let has_default = !param.option_list.is_empty();
        // if has_default {
        //     let size = param.option_list.len() as u8;
        //     for v in &param.option_list {
        //         match &v.expr {
        //             Some(expr) => {
        //                 self.compile(expr, None);
        //             }
        //             None => {
        //                 self.push_load_const(PyObject::None(false));
        //             }
        //         }
        //     }
        //     self.push_op(OpCode::BuildTuple(size));
        // }

        // // キーワード引数のデフォルト値の設定
        // let has_kw_default = param.named_list.iter().any(|v| v.expr.is_some());
        // if has_kw_default {
        //     let mut name_list: Vec<&str> = vec![];
        //     for v in &param.named_list {
        //         match &v.expr {
        //             Some(expr) => {
        //                 self.compile(expr, None);
        //                 name_list.push(v.identifier.value);
        //             }
        //             None => (),
        //         }
        //     }
        //     self.push_load_const(PyObject::SmallTuple {
        //         children: name_list
        //             .iter()
        //             .map(|v| PyObject::new_string(v.to_string(), false))
        //             .collect(),
        //         add_ref: false,
        //     });
        //     let size = name_list.len() as u8;
        //     self.push_op(OpCode::BuildConstKeyMap(size));
        // }

        // アノテーションは未実装
        // クロージャは未実装

        // コードオブジェクトの読み込み
        self.push_load_const(py_code);
        // 関数名の読み込み
        match function_name_prefix {
            Some(prefix) => {
                self.push_load_const(PyObject::new_string(format!("{}{}", prefix, name), false));
            }
            None => {
                self.push_load_const(PyObject::new_string(name.to_string(), false));
            }
        }
        // 関数作成と収納
        // let make_flag = (has_default as u8) | ((has_kw_default as u8) << 1);
        let make_flag = 0;
        self.push_op(OpCode::MakeFunction(make_flag));
        let p = (**self.context_stack.last().unwrap())
            .borrow_mut()
            .declare_variable(&name);
        self.push_op(OpCode::StoreName(p));
    }

    fn push_op(&self, op: OpCode) {
        self.byte_operations.borrow_mut().push(op);
    }

    fn push_load_const(&self, value: PyObject) -> u8 {
        let position = self.context_stack.last().unwrap().borrow().const_len() as u8;
        (**self.context_stack.last().unwrap())
            .borrow_mut()
            .push_const(value);
        self.push_op(OpCode::LoadConst(position));
        position
    }

    fn push_load_var(&self, value: &String) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global | VariableScope::NotDefined => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::LoadName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::LoadGlobal(p));
                }
            }
            VariableScope::Local => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(value);
                self.push_op(OpCode::LoadFast(p));
            }
            VariableScope::Instance => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(&"self".to_string());
                self.push_op(OpCode::LoadFast(p));
                let p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(value);
                self.push_op(OpCode::LoadAttr(p));
            }
        }
    }

    fn push_store_var(&self, value: &String) {
        let scope = self
            .context_stack
            .last()
            .unwrap()
            .borrow()
            .check_variable_scope(value);
        match scope {
            VariableScope::Global | VariableScope::NotDefined => {
                if self.context_stack.last().unwrap().borrow().is_global() {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::StoreName(p));
                } else {
                    let p = (**self.context_stack.last().unwrap())
                        .borrow_mut()
                        .register_or_get_name(value);
                    self.push_op(OpCode::StoreGlobal(p));
                }
            }
            VariableScope::Local => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(value);
                self.push_op(OpCode::StoreFast(p));
            }
            VariableScope::Instance => {
                let p = self
                    .context_stack
                    .last()
                    .unwrap()
                    .borrow()
                    .get_local_variable(&"self".to_string());
                self.push_op(OpCode::LoadFast(p));
                let p = (**self.context_stack.last().unwrap())
                    .borrow_mut()
                    .register_or_get_name(value);
                self.push_op(OpCode::StoreAttr(p));
            }
        }
    }

    fn gen_jump_label(&self) -> u32 {
        let key = *self.jump_label_key_index.borrow();
        *self.jump_label_key_index.borrow_mut() += 1;
        key
    }

    // 今の次の位置にラベル位置を合わせる
    fn set_jump_label_value(&self, key: u32) {
        let index = self.byte_operations.borrow().len() as u8;
        // 1命令あたり2バイトなので2倍
        self.jump_label_table.borrow_mut().insert(key, index * 2);
    }

    fn set_jump_label_value_offset(&self, key: u32, offset: u8) {
        let index = (self.byte_operations.borrow().len() as u8) - offset;
        // 1命令あたり2バイトなので2倍
        self.jump_label_table.borrow_mut().insert(key, index * 2);
    }
}

impl<'a, 'b> ByteCompiler<'a, 'b> {
    pub fn resolve_references(&self) -> Vec<ByteCode> {
        let opcode_list = self.byte_operations.borrow();

        let result = opcode_list
            .iter()
            .map(|v| v.resolve(&self.jump_label_table.borrow()))
            .collect();

        result
    }
}
