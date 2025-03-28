use std::collections::{HashMap, HashSet};

use anyhow::{Ok, Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::{Symbol, TypeTraverseHistory},
    type_system::{
        type_scheme::TypeScheme,
        types::Type,
        unification::{Equations, get_equation},
        unification::{add_equation, unify},
    },
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
enum NormalizeError {
    #[error("Cyclic type reference occur")]
    CyclicTypeReference,
    #[error("Unresolved type")]
    UnresolvedType,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TypeEnvironment {
    variable_types: HashMap<Symbol, TypeScheme>,
    equations: Equations,
}

impl TypeEnvironment {
    pub fn get_variable_type(&self, variable_name: &Symbol) -> Result<Type> {
        if let Some(type_scheme) = self.variable_types.get(variable_name).cloned() {
            return Ok(type_scheme.instantiate());
        }

        bail!(NormalizeError::UnresolvedType);
    }

    pub fn get_unbound_variables<T: Iterator<Item = Symbol>>(
        &self,
        variables: T,
    ) -> HashSet<Symbol> {
        let mut free_variables = HashSet::from_iter(variables);
        self.variable_types.keys().for_each(|variable_name| {
            free_variables.remove(variable_name);
        });
        free_variables
    }

    pub fn substitute_variable(
        self,
        variable_name: Symbol,
        type_scheme: TypeScheme,
    ) -> Result<Self> {
        let mut variable_types = self.variable_types.clone();
        variable_types.insert(variable_name, type_scheme);

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
            Type::List(t) => Ok(Type::List(self.normalize_type(visited, *t)?.into())),
            variable @ Type::Variable { .. } => {
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
