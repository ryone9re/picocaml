use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    adapter::Symbol,
    type_system::{types::Type, unification::Equations},
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
        let equations = self.equations.add(type1, type2);

        Self {
            variable_types: self.variable_types,
            equations,
        }
    }

    pub fn unify_equations(self) -> Result<Self> {
        let equations = self.equations.unify()?;

        Ok(Self {
            variable_types: self.variable_types,
            equations,
        })
    }

    pub fn normalize_type(&self, visited: HashSet<Type>, t: Type) -> Result<Type> {
        let normalized = match t {
            Type::Base(base_type) => Type::Base(base_type),
            variable @ Type::Variable { name: _ } => {
                if visited.contains(&variable) {
                    bail!(NormalizeError::CyclicTypeReference);
                }
                self.equations
                    .get(variable)
                    .ok_or(anyhow!(NormalizeError::UnresolvedType))?
            }
            Type::Function { domain, range } => {
                let domain = self.normalize_type(visited.clone(), *domain)?;
                let range = self.normalize_type(visited.clone(), *range)?;

                Type::Function {
                    domain: domain.into(),
                    range: range.into(),
                }
            }
        };

        println!("{:?}", normalized);

        Ok(normalized)
    }
}
