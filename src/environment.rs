use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub(crate) fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
    pub(crate) fn get(&self, name: &str) -> InterpreterResult<&Value> {
        self.values
            .get(name)
            .ok_or_else(|| InterpreterError::undefined_variable_error(String::from(name)))
    }
}
