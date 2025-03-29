use std::collections::HashMap;

use anyhow::{Ok, Result};

use crate::{adapter::Symbol, syntax::value::Value};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Environment {
    variables: HashMap<Symbol, Value>,
}

impl Environment {
    pub fn bind(self, variable: Symbol, value: Value) -> Result<Self> {
        let mut new = self.clone();
        new.variables.insert(variable, value);
        Ok(new)
    }

    pub fn get(&self, variable: &Symbol) -> Option<Value> {
        self.variables.get(variable).cloned()
    }
}
