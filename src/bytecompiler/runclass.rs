use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::bytecode::{calc_stack_size, OpCode};
use crate::executioncontext::{ClassContext, ExecutionContext, PyContext};
use crate::pyobject::PyObject;

use super::ByteCompiler;

// pub fn run_class<'ctx, 'value, 'cpl>(
//     file_name: &String,
//     code_name: &String,
//     member_list: &'value Vec<Member>,
//     outer_compiler: &'cpl ByteCompiler<'ctx, 'value>,
//     source: &'value str,
// ) -> PyObject {
//     let py_context = Rc::new(RefCell::new(PyContext {
//         outer: outer_compiler.context_stack.last().unwrap().clone(),
//         constant_list: vec![],
//         name_list: vec![],
//         name_map: HashMap::new(),
//         local_variables: vec![],
//     }));

//     let class_context = Rc::new(RefCell::new(ClassContext {
//         outer: py_context.clone(),
//         instance_variables: vec![],
//     }));

//     let mut compiler = ByteCompiler {
//         byte_operations: RefCell::new(vec![]),
//         context_stack: vec![class_context.clone()],
//         jump_label_table: RefCell::new(HashMap::new()),
//         jump_label_key_index: RefCell::new(*outer_compiler.jump_label_key_index.borrow()),
//         default_scope_stack: vec![],
//         break_label_table: HashMap::new(),
//         continue_label_table: HashMap::new(),
//         source,
//     };

//     // __module__ = __name__
//     let p = (compiler.context_stack.last().unwrap())
//         .borrow_mut()
//         .register_or_get_name(&"__name__".to_string());
//     compiler.push_op(OpCode::LoadName(p));

//     let p = (compiler.context_stack.last().unwrap())
//         .borrow_mut()
//         .register_or_get_name(&"__module__".to_string());
//     compiler.push_op(OpCode::StoreName(p));

//     // __qualname__ = "Hoge"
//     compiler.push_load_const(PyObject::new_string("Hoge".to_string(), false));
//     let p = (compiler.context_stack.last().unwrap())
//         .borrow_mut()
//         .register_or_get_name(&"__qualname__".to_string());
//     compiler.push_op(OpCode::StoreName(p));

//     // メンバの分類
//     let mut instance_variable_declaration_list: Vec<&Vec<VariableDeclaration>> = vec![];
//     let mut primary_constructor: Option<&Member> = None;
//     let mut method_declaration_list: Vec<&Member> = vec![];
//     for member in member_list {
//         match member {
//             Member::VariableDecl { decl_list } => {
//                 instance_variable_declaration_list.push(&decl_list);
//                 // インスタンス変数を登録
//                 for decl in decl_list {
//                     class_context
//                         .borrow_mut()
//                         .declare_variable(&decl.identifier.value.to_string());
//                 }
//             }
//             Member::MethodImpl { signature, body: _ } => {
//                 if signature.name.value == code_name {
//                     primary_constructor = Some(member);
//                 } else {
//                     method_declaration_list.push(member);
//                 }
//             }
//             Member::ConstructorImpl {
//                 signature: _,
//                 body: _,
//             } => {
//                 primary_constructor = Some(member);
//             }
//         }
//     }

//     let dummy_constructor = Member::ConstructorImpl {
//         signature: ConstructorSignature {
//             name: None,
//             param: FunctionParamSignature {
//                 normal_list: vec![],
//                 option_list: vec![],
//                 named_list: vec![],
//             },
//         },
//         body: Box::new(Node::EmptyStatement),
//     };
//     if !instance_variable_declaration_list.is_empty() && primary_constructor.is_none() {
//         primary_constructor = Some(&dummy_constructor);
//     }

//     if let Some(method) = primary_constructor {
//         compile_constructor(
//             &mut compiler,
//             method,
//             code_name,
//             instance_variable_declaration_list,
//         );
//     }

//     for method in method_declaration_list {
//         compile_method(&mut compiler, &method, code_name)
//     }

//     // 終わり
//     compiler.push_load_const(PyObject::None(false));
//     compiler.push_op(OpCode::ReturnValue);

//     compiler.context_stack.pop();
//     drop(class_context);

//     // outer_compilerへの情報の復帰
//     *outer_compiler.jump_label_key_index.borrow_mut() = *compiler.jump_label_key_index.borrow();

//     // PyCodeの作成
//     let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
//     let operation_list = compiler.resolve_references();

//     let py_context = Rc::try_unwrap(py_context).ok().unwrap().into_inner();

//     PyObject::Code {
//         file_name: file_name.to_string(),
//         code_name: code_name.to_string(),
//         num_args: 0,
//         num_pos_only_args: 0,
//         num_kw_only_args: 0,
//         num_locals: 0,
//         stack_size,
//         operation_list,
//         constant_list: Box::new(PyObject::SmallTuple {
//             children: py_context.constant_list,
//             add_ref: false,
//         }),
//         name_list: Box::new(PyObject::SmallTuple {
//             children: py_context.name_list,
//             add_ref: false,
//         }),
//         local_list: Box::new(PyObject::SmallTuple {
//             children: vec![],
//             add_ref: false,
//         }),
//         add_ref: false,
//     }
// }

// fn compile_method<'ctx, 'value, 'cpl>(
//     compiler: &'cpl mut ByteCompiler<'ctx, 'value>,
//     node: &'value Member,
//     class_name: &'value str,
// ) {
//     if let Member::MethodImpl { signature, body } = node {
//         let prefix = format!("{}{}", class_name, ".");
//         compiler.comiple_declare_function(
//             &signature.name.value.to_string(),
//             &signature.param,
//             &body,
//             Some(prefix),
//             Some(&"self".to_string()),
//             |_| (),
//         );
//     }
// }

// fn compile_constructor<'ctx, 'value, 'cpl>(
//     compiler: &'cpl mut ByteCompiler<'ctx, 'value>,
//     node: &'value Member,
//     class_name: &'value str,
//     instance_variable_declaration_list: Vec<&'value Vec<VariableDeclaration>>,
// ) {
//     let preface = |compiler: &mut ByteCompiler<'ctx, 'value>| {
//         for decl_list in instance_variable_declaration_list {
//             for decl in decl_list {
//                 match &decl.expr {
//                     Some(expr) => compiler.compile(&*expr, None),
//                     None => {
//                         compiler.push_load_const(PyObject::None(false));
//                     }
//                 }
//                 let p = compiler
//                     .context_stack
//                     .last()
//                     .unwrap()
//                     .borrow()
//                     .get_local_variable(&"self".to_string());
//                 compiler.push_op(OpCode::LoadFast(p));

//                 let p = (**compiler.context_stack.last().unwrap())
//                     .borrow_mut()
//                     .register_or_get_name(&decl.identifier.value.to_string());
//                 compiler.push_op(OpCode::StoreAttr(p));
//             }
//         }
//     };
//     match node {
//         Member::MethodImpl { signature, body } => {
//             let prefix = format!("{}{}", class_name, ".");
//             compiler.comiple_declare_function(
//                 &"__init__".to_string(),
//                 &signature.param,
//                 &body,
//                 Some(prefix),
//                 Some(&"self".to_string()),
//                 preface,
//             );
//         }
//         Member::ConstructorImpl { signature, body } => {
//             let prefix = format!("{}{}", class_name, ".");
//             compiler.comiple_declare_function(
//                 &"__init__".to_string(),
//                 &signature.param,
//                 &body,
//                 Some(prefix),
//                 Some(&"self".to_string()),
//                 preface,
//             );
//         }
//         _ => (),
//     }
// }
