use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::pyobject::PyObject;

pub struct GlobalContext {
    pub constant_list: Vec<PyObject>,
    pub name_list: Vec<PyObject>,
    pub name_map: HashMap<String, u8>,
    pub global_variables: Vec<String>,
}
pub struct PyContext<'ctx> {
    pub outer: Rc<RefCell<dyn ExecutionContext + 'ctx>>,
    pub constant_list: Vec<PyObject>,
    pub name_list: Vec<PyObject>,
    pub name_map: HashMap<String, u8>,
    pub local_variables: Vec<String>,
}
pub struct BlockContext<'ctx> {
    pub outer: Rc<RefCell<dyn ExecutionContext + 'ctx>>,
    pub variables: Vec<String>,
}

pub trait ExecutionContext {
    fn push_const(&mut self, value: PyObject);
    fn const_len(&self) -> usize;
    fn declare_variable(&mut self, symbol: &String) -> u8;
    fn get_local_variable(&self, symbol: &String) -> u8;
    fn check_variable_scope(&self, symbol: &String) -> VariableScope;
    fn register_or_get_name(&mut self, name: &String) -> u8;
    fn is_global(&self) -> bool;
}

impl ExecutionContext for GlobalContext {
    fn push_const(&mut self, value: PyObject) {
        self.constant_list.push(value);
    }

    fn const_len(&self) -> usize {
        self.constant_list.len()
    }

    fn declare_variable(&mut self, symbol: &String) -> u8 {
        // グローバル変数の定義
        let position = self.name_list.len() as u8;
        let obj = PyObject::new_string(symbol.to_string(), false);
        self.name_list.push(obj);
        self.name_map.insert(symbol.to_string(), position);
        self.global_variables.push(symbol.clone());
        position
    }

    fn get_local_variable(&self, _symbol: &String) -> u8 {
        panic!("Not Implemented");
    }

    fn check_variable_scope(&self, symbol: &String) -> VariableScope {
        if self.global_variables.contains(symbol) {
            VariableScope::Global
        } else {
            VariableScope::NotDefined
        }
    }

    fn register_or_get_name(&mut self, name: &String) -> u8 {
        match self.name_map.get(name) {
            Some(v) => *v,
            None => {
                let position = self.name_list.len() as u8;
                let obj = PyObject::new_string(name.to_string(), false);
                self.name_list.push(obj);
                self.name_map.insert(name.clone(), position);
                position
            }
        }
    }

    fn is_global(&self) -> bool {
        true
    }
}

impl<'ctx> ExecutionContext for PyContext<'ctx> {
    fn push_const(&mut self, value: PyObject) {
        self.constant_list.push(value);
    }

    fn const_len(&self) -> usize {
        self.constant_list.len()
    }

    fn declare_variable(&mut self, symbol: &String) -> u8 {
        // ローカル変数の定義
        let position = self.name_list.len() as u8;
        let obj = PyObject::new_string(symbol.to_string(), false);
        self.name_list.push(obj);
        self.name_map.insert(symbol.to_string(), position);

        self.local_variables.push(symbol.clone());
        position
    }

    fn get_local_variable(&self, symbol: &String) -> u8 {
        self.local_variables
            .iter()
            .position(|v| v == symbol)
            .unwrap() as u8
    }

    fn check_variable_scope(&self, symbol: &String) -> VariableScope {
        self.outer.borrow().check_variable_scope(symbol)
    }

    fn register_or_get_name(&mut self, name: &String) -> u8 {
        match self.name_map.get(name) {
            Some(v) => *v,
            None => {
                let position = self.name_list.len() as u8;
                let obj = PyObject::new_string(name.to_string(), false);
                self.name_list.push(obj);
                self.name_map.insert(name.clone(), position);
                position
            }
        }
    }

    fn is_global(&self) -> bool {
        false
    }
}

impl<'ctx> ExecutionContext for BlockContext<'ctx> {
    fn push_const(&mut self, value: PyObject) {
        self.outer.borrow_mut().push_const(value);
    }

    fn const_len(&self) -> usize {
        self.outer.borrow().const_len()
    }

    fn declare_variable(&mut self, symbol: &String) -> u8 {
        // ブロック内ローカル変数の定義
        self.variables.push(symbol.clone());
        self.outer.borrow_mut().declare_variable(symbol)
    }

    fn get_local_variable(&self, symbol: &String) -> u8 {
        self.outer.borrow_mut().get_local_variable(symbol)
    }

    fn check_variable_scope(&self, symbol: &String) -> VariableScope {
        if self.variables.contains(symbol) {
            VariableScope::Local
        } else {
            self.outer.borrow().check_variable_scope(symbol)
        }
    }

    fn register_or_get_name(&mut self, name: &String) -> u8 {
        self.outer.borrow_mut().register_or_get_name(name)
    }

    fn is_global(&self) -> bool {
        false
    }
}

pub enum VariableScope {
    Global,
    Local,
    NotDefined,
}
