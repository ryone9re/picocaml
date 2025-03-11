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
    pub fn bind_variable(self, variable: Symbol, value: Value) -> Result<Self> {
        let new_environment = self.environment.bind(variable, value)?;
        let new_type_environment = self.type_environment.clone();

        Ok(Self {
            environment: new_environment,
            type_environment: new_type_environment,
        })
    }

    pub fn get_variable_value(&self, variable: &Symbol) -> Option<Value> {
        self.environment.get(variable)
    }
}
