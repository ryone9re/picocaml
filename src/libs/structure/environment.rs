use std::collections::HashMap;

use anyhow::Result;

use crate::{adapter::Symbol, value::Value};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Environment {
    variables: HashMap<Symbol, Value>,
}

impl Environment {
    pub fn assign_variable(self, variable: Symbol, value: Value) -> Result<Self> {
        let mut new = self.clone();
        new.variables.insert(variable, value);
        Ok(new)
    }

    pub fn get_variable_value(&self, variable: &Symbol) -> Option<Value> {
        self.variables.get(variable).cloned()
    }
}
