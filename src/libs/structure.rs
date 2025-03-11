use std::collections::HashMap;

use crate::{adapter::Symbol, value::Value};
use anyhow::Result;

type Environment = HashMap<Symbol, Value>;

type TypeEnvironment = HashMap<Symbol, ()>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Structure {
    environment: Environment,
    type_environment: TypeEnvironment,
}

impl Structure {
    pub fn assign_variable(self, variable: Symbol, value: Value) -> Result<Structure> {
        let mut new_structure = self.clone();
        new_structure.environment.insert(variable, value);
        Ok(new_structure)
    }

    pub fn get_variable_value(&self, variable: &Symbol) -> Option<Value> {
        self.environment.get(variable).cloned()
    }
}
