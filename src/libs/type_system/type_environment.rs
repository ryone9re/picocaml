use std::collections::HashMap;

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
}

impl TypeEnvironment {
    pub fn is_empty(&self) -> bool {
        self.variable_types.is_empty()
    }

    pub fn get_variable_type(&self, variable_name: &Symbol) -> Option<Type> {
        self.variable_types.get(variable_name).cloned()
    }
}
