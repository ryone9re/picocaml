mod environment;
mod type_environment;

use crate::{adapter::Symbol, value::Value};
use anyhow::Result;
use environment::Environment;
use type_environment::TypeEnvironment;

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
