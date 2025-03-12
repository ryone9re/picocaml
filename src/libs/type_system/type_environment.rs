use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::adapter::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    Integer,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Base(BaseType),
    Variable { name: Symbol },
    Function { domain: Box<Type>, range: Box<Type> },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TypeEnvironment {
    variable_types: HashMap<Symbol, Type>,
    constraints: HashSet<(Type, Type)>,
}

impl TypeEnvironment {
    pub fn get_variable_type(&self, variable_name: &Symbol) -> Option<Type> {
        self.variable_types.get(variable_name).cloned()
    }

    pub fn substitute_variable(self, variable_name: Symbol, t: Type) -> Result<Self> {
        let mut variable_types = self.variable_types.clone();
        variable_types.insert(variable_name, t);

        Ok(Self {
            variable_types,
            constraints: self.constraints,
        })
    }

    pub fn add_constraint(self, type1: Type, type2: Type) -> Self {
        let mut constraints = self.constraints.clone();
        constraints.insert((type1, type2));

        Self {
            variable_types: self.variable_types,
            constraints,
        }
    }
}
