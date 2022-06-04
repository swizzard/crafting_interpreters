use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub(crate) struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub(crate) fn new(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::default(),
        }
    }
    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
    pub(crate) fn get(&self, name: &str) -> InterpreterResult<Value> {
        match self.values.get(name) {
            Some(v) => Ok(v.clone()),
            None => match &self.enclosing {
                None => Err(InterpreterError::undefined_variable_error(String::from(
                    name,
                ))),
                Some(e) => Ok(e.borrow().get(name)?.clone()),
            },
        }
    }
    pub(crate) fn assign(&mut self, name: &str, value: Value) -> InterpreterResult<Value> {
        if self.values.contains_key(name) {
            self.define(name.to_string(), value.clone());
            Ok(value)
        } else {
            match self.enclosing.as_ref() {
                Some(e) => e.borrow_mut().assign(name, value),
                None => Err(InterpreterError::undefined_variable_error(name.into())),
            }
        }
    }
}
