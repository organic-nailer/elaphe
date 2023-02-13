use std::collections::HashMap;

use crate::pyobject::PyObject;

pub struct GlobalContext<'value> {
    pub constant_list: Vec<PyObject<'value>>,
    pub name_list: Vec<PyObject<'value>>,
    pub name_map: HashMap<&'value str, u8>,
    pub global_variables: Vec<&'value str>,
}
pub struct PyContext<'ctx, 'value> {
    pub outer: &'ctx dyn ExecutionContext<'value>,
    pub constant_list: Vec<PyObject<'value>>,
    pub name_list: Vec<PyObject<'value>>,
    pub name_map: HashMap<&'value str, u8>,
    pub local_variables: Vec<&'value str>,
}
pub struct BlockContext<'ctx, 'value> {
    pub outer: &'ctx mut dyn ExecutionContext<'value>,
    pub variables: Vec<&'value str>,
}

pub trait ExecutionContext<'value> {
    fn push_const(&mut self, value: PyObject<'value>);
    fn const_len(&self) -> usize;
    // return: (name_position, variable_position)
    fn declare_variable(&mut self, symbol: &'value str) -> u8;
    fn get_local_variable(&self, symbol: &'value str) -> u8;
    fn check_variable_scope(&self, symbol: &str) -> VariableScope;
    fn register_or_get_name(&mut self, name: &'value str) -> u8;
    fn is_global(&self) -> bool;
}

impl<'value> ExecutionContext<'value> for GlobalContext<'value> {
    fn push_const(&mut self, value: PyObject<'value>) {
        self.constant_list.push(value);
    }

    fn const_len(&self) -> usize {
        self.constant_list.len()
    }

    fn declare_variable(&mut self, symbol: &'value str) -> u8 {
        // グローバル変数の定義
        let position = self.name_list.len() as u8;
        let obj = PyObject::new_string(symbol, false);
        self.name_list.push(obj);
        self.name_map.insert(symbol, position);
        self.global_variables.push(symbol);
        position
    }

    fn get_local_variable(&self, symbol: &'value str) -> u8 {
        panic!("Not Implemented");
    }

    fn check_variable_scope(&self, symbol: &str) -> VariableScope {
        if self.global_variables.contains(&symbol) {
            VariableScope::Global
        } else {
            VariableScope::NotDefined
        }
    }

    fn register_or_get_name(&mut self, name: &'value str) -> u8 {
        match self.name_map.get(name) {
            Some(v) => *v,
            None => {
                let position = self.name_list.len() as u8;
                let obj = PyObject::new_string(name, false);
                self.name_list.push(obj);
                self.name_map.insert(name, position);
                position
            }
        }
    }

    fn is_global(&self) -> bool { true }
}

impl<'ctx, 'value> ExecutionContext<'value> for PyContext<'ctx, 'value> {
    fn push_const(&mut self, value: PyObject<'value>) {
        self.constant_list.push(value);
    }

    fn const_len(&self) -> usize {
        self.constant_list.len()
    }

    fn declare_variable(&mut self, symbol: &'value str) -> u8 {
        // ローカル変数の定義
        let position = self.name_list.len() as u8;
        let obj = PyObject::new_string(symbol, false);
        self.name_list.push(obj);
        self.name_map.insert(symbol, position);

        self.local_variables.push(symbol);
        position
    }

    fn get_local_variable(&self, symbol: &'value str) -> u8 {
        self.local_variables.iter().position(|v| *v == symbol).unwrap() as u8
    }

    fn check_variable_scope(&self, symbol: &str) -> VariableScope {
        self.outer.check_variable_scope(symbol)
    }

    fn register_or_get_name(&mut self, name: &'value str) -> u8 {
        match self.name_map.get(name) {
            Some(v) => *v,
            None => {
                let position = self.name_list.len() as u8;
                let obj = PyObject::new_string(name, false);
                self.name_list.push(obj);
                self.name_map.insert(name, position);
                position
            }
        }
    }

    fn is_global(&self) -> bool { false }
}

impl<'ctx, 'value> ExecutionContext<'value> for BlockContext<'ctx, 'value> {
    fn push_const(&mut self, value: PyObject<'value>) {
        self.outer.push_const(value);
    }

    fn const_len(&self) -> usize {
        self.outer.const_len()
    }

    fn declare_variable(&mut self, symbol: &'value str) -> u8 {
        // ブロック内ローカル変数の定義
        self.variables.push(symbol);
        self.outer.declare_variable(symbol)
    }

    fn get_local_variable(&self, symbol: &'value str) -> u8 {
        self.outer.get_local_variable(symbol)
    }

    fn check_variable_scope(&self, symbol: &str) -> VariableScope {
        if self.variables.contains(&symbol) {
            VariableScope::Local
        } else {
            self.outer.check_variable_scope(symbol)
        }
    }

    fn register_or_get_name(&mut self, name: &'value str) -> u8 {
        self.outer.register_or_get_name(name)
    }

    fn is_global(&self) -> bool { false }
}

pub enum VariableScope {
    Global,
    Local,
    NotDefined,
}
