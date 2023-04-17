use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::bytecode::{calc_stack_size, OpCode};
use crate::executioncontext::{ExecutionContext, GlobalContext};
use crate::parser::node::LibraryDeclaration;
use crate::pyobject::PyObject;

use super::ByteCompiler;

pub fn run_root<'value>(
    file_name: &String,
    root_node: &'value LibraryDeclaration<'value>,
    source: &'value str,
) -> PyObject {
    let global_context = Rc::new(RefCell::new(GlobalContext {
        constant_list: vec![],
        name_list: vec![],
        name_map: HashMap::new(),
        global_variables: vec![],
    }));

    let mut compiler = ByteCompiler {
        byte_operations: RefCell::new(vec![]),
        context_stack: vec![global_context.clone()],
        jump_label_table: RefCell::new(HashMap::new()),
        jump_label_key_index: RefCell::new(0),
        default_scope_stack: vec![],
        break_label_table: HashMap::new(),
        continue_label_table: HashMap::new(),
        source,
    };

    // 0番目の定数にNoneを追加
    (*global_context)
        .borrow_mut()
        .push_const(PyObject::None(false));

    for node in &root_node.import_list {
        compiler.compile_import(node);
    }

    for node in &root_node.top_level_declaration_list {
        compiler.compile_stmt(&node, None);
    }

    // main関数を実行
    let main_position = (*global_context)
        .borrow_mut()
        .register_or_get_name(&"main".to_string());
    compiler.push_op(OpCode::LoadName(main_position));
    compiler.push_op(OpCode::CallFunction(0));
    compiler.push_op(OpCode::PopTop);
    compiler.push_op(OpCode::LoadConst(0));
    compiler.push_op(OpCode::ReturnValue);

    let stack_size = calc_stack_size(&compiler.byte_operations.borrow()) as u32;
    let operation_list = compiler.resolve_references();

    compiler.context_stack.pop();

    // 所有が1箇所しかないはずなのでRcの外に出す
    let global_context = Rc::try_unwrap(global_context).ok().unwrap().into_inner();

    let constant_list = PyObject::SmallTuple {
        children: global_context.constant_list,
        add_ref: false,
    };
    let name_list = PyObject::SmallTuple {
        children: global_context.name_list,
        add_ref: false,
    };

    PyObject::Code {
        file_name: file_name.to_string(),
        code_name: "<module>".to_string(),
        num_args: 0,
        num_pos_only_args: 0,
        num_kw_only_args: 0,
        num_locals: 0,
        stack_size,
        operation_list,
        constant_list: Box::new(constant_list),
        name_list: Box::new(name_list),
        local_list: Box::new(PyObject::SmallTuple {
            children: vec![],
            add_ref: false,
        }),
        add_ref: true,
    }
}
