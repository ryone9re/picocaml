use std::collections::HashSet;

use anyhow::{Result, bail};
use thiserror::Error;

use crate::type_system::types::Type;

#[derive(Debug, Error)]
enum UnificationError {
    #[error("Unification impossible")]
    Impossible,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Equations {
    equations: HashSet<(Type, Type)>,
}

impl Equations {
    pub fn get(&self, variable: Type) -> Option<Type> {
        let found_lhs = self
            .equations
            .iter()
            .filter(|(t1, _)| *t1 == variable)
            .last()
            .cloned();
        if let Some((_, found)) = found_lhs {
            return Some(found);
        }

        let found_rhs = self
            .equations
            .iter()
            .filter(|(_, t2)| *t2 == variable)
            .last()
            .cloned();
        if let Some((found, _)) = found_rhs {
            return Some(found);
        }

        None
    }

    pub fn add(self, type1: Type, type2: Type) -> Self {
        let mut equations = self.equations.clone();
        equations.insert((type1, type2));

        Self { equations }
    }

    pub fn substitute_all_variable(self, target_type: Type, new_type: Type) -> Result<Self> {
        let equations = self
            .equations
            .into_iter()
            .map(|(t1, t2)| {
                let type1 = if t1 == target_type {
                    new_type.clone()
                } else {
                    t1
                };
                let type2 = if t2 == target_type {
                    new_type.clone()
                } else {
                    t2
                };

                (type1, type2)
            })
            .collect();

        Ok(Self { equations })
    }

    pub fn unify(self) -> Result<Self> {
        let Ok(result) = self.unify1()?.unify2()?.unify3() else {
            bail!(UnificationError::Impossible);
        };

        if result
            .equations
            .iter()
            .any(|(t1, t2)| matches!(t1, Type::Base(_)) && matches!(t2, Type::Base(_)) && t1 != t2)
        {
            bail!(UnificationError::Impossible);
        }

        Ok(result)
    }

    // (EU{(p,p)},S) => (E,S)
    fn unify1(self) -> Result<Self> {
        let equations = self
            .equations
            .into_iter()
            .filter(|(type1, type2)| type1 != type2)
            .collect();

        Ok(Self { equations })
    }

    // (EU{(a,p)},S) => ([p/a]E,{(a,p)}U[p/a]S) ただしa∉FTV(p)
    fn unify2(self) -> Result<Self> {
        let mut e = self.clone();
        let mut substitution = Self::default();

        loop {
            if e.equations.is_empty() {
                break;
            }

            if let Some((t1, t2)) = e.equations.iter().next().cloned() {
                e.equations.remove(&(t1.clone(), t2.clone()));
                e = e.substitute_all_variable(t1.clone(), t2.clone())?;

                substitution = substitution.substitute_all_variable(t1.clone(), t2.clone())?;
                substitution = substitution.add(t1.clone(), t2.clone());
            }
        }

        Ok(substitution)
    }

    // (EU{(p1->r1,p2->r2)},S) => (EU{(p1,p2),(r1,r2)},S)
    fn unify3(self) -> Result<Self> {
        let equations = self
            .equations
            .into_iter()
            .flat_map(|(type1, type2)| match (type1, type2) {
                (
                    Type::Function {
                        domain: domain1,
                        range: range1,
                    },
                    Type::Function {
                        domain: domain2,
                        range: range2,
                    },
                ) => vec![(*domain1, *domain2), (*range1, *range2)],
                pair => vec![pair],
            })
            .collect();

        Ok(Self { equations })
    }
}
