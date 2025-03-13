use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::{Symbol, TypeTraverseHistory},
    type_system::{
        types::Type,
        unification::{Equations, get_equation},
    },
};

use super::unification::{add_equation, unify};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
enum NormalizeError {
    #[error("Cyclic type reference occur")]
    CyclicTypeReference,
    #[error("Unresolved type")]
    UnresolvedType,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TypeEnvironment {
    variable_types: HashMap<Symbol, Type>,
    equations: Equations,
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
            equations: self.equations,
        })
    }

    pub fn add_equation(self, type1: Type, type2: Type) -> Self {
        let equations = add_equation(self.equations, type1, type2);

        Self {
            variable_types: self.variable_types,
            equations,
        }
    }

    pub fn unify_equations(self) -> Result<Self> {
        let equations = unify(self.equations.clone(), Equations::new())?;

        Ok(Self {
            variable_types: self.variable_types,
            equations,
        })
    }

    pub fn normalize_type(&self, mut visited: TypeTraverseHistory, t: Type) -> Result<Type> {
        match t {
            Type::Base(base_type) => Ok(Type::Base(base_type)),
            variable @ Type::Variable { name: _ } => {
                if visited.contains(&variable) {
                    bail!(NormalizeError::CyclicTypeReference);
                }
                visited.insert(variable.clone());
                get_equation(&self.equations, variable)
                    .ok_or(anyhow!(NormalizeError::UnresolvedType))
            }
            Type::Function { domain, range } => Ok(Type::Function {
                domain: self.normalize_type(visited.clone(), *domain)?.into(),
                range: self.normalize_type(visited.clone(), *range)?.into(),
            }),
        }
    }
}
