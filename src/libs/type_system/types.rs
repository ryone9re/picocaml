use std::{collections::HashSet, fmt::Display};

use crate::adapter::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    Integer,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Base(BaseType),
    List(Box<Type>),
    Variable { name: Symbol },
    Function { domain: Box<Type>, range: Box<Type> },
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Type {
    pub fn apply_substitution(
        self,
        target_variable_name: Symbol,
        new_variable_name: Symbol,
    ) -> Self {
        match self {
            Type::Variable { name } if name == target_variable_name => Type::Variable {
                name: new_variable_name,
            },
            Type::Function { domain, range } => Type::Function {
                domain: domain
                    .apply_substitution(target_variable_name.clone(), new_variable_name.clone())
                    .into(),
                range: range
                    .apply_substitution(target_variable_name.clone(), new_variable_name.clone())
                    .into(),
            },
            t => t,
        }
    }

    pub fn apply_substitution_for_type(self, target_variable_name: Symbol, new_type: Type) -> Self {
        match self {
            Type::Variable { name } if name == target_variable_name => new_type,
            Type::Function { domain, range } => Type::Function {
                domain: domain
                    .apply_substitution_for_type(target_variable_name.clone(), new_type.clone())
                    .into(),
                range: range
                    .apply_substitution_for_type(target_variable_name.clone(), new_type.clone())
                    .into(),
            },
            t => t,
        }
    }
}

pub fn free_type_variables(t: Type) -> HashSet<Symbol> {
    match t {
        Type::Variable { name } => HashSet::from_iter(vec![name]),
        Type::Function { domain, range } => free_type_variables(*domain)
            .union(&free_type_variables(*range))
            .cloned()
            .collect(),
        Type::List(element_type) => free_type_variables(*element_type),
        Type::Base(_) => HashSet::new(),
    }
}
