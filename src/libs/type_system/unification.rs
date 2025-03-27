use std::collections::HashSet;

use anyhow::{Result, bail};
use thiserror::Error;

use crate::{
    adapter::{Symbol, SymbolTraverseHistory},
    type_system::types::Type,
};

#[derive(Debug, Error)]
enum UnificationError {
    #[error("Unification impossible")]
    Impossible,
    #[error("Circular reference occur")]
    CircularReference,
}

pub type Equations = HashSet<(Type, Type)>;

pub fn add_equation(equations: Equations, type1: Type, type2: Type) -> Equations {
    let mut equations = equations.clone();
    equations.insert((type1, type2));
    equations
}

pub fn get_equation(equations: &Equations, t: Type) -> Option<Type> {
    get_equation_internal(equations, t, &mut SymbolTraverseHistory::new())
}

fn get_equation_internal(
    equations: &Equations,
    t: Type,
    visited: &mut SymbolTraverseHistory,
) -> Option<Type> {
    let Type::Variable { name } = t.clone() else {
        return Some(t);
    };

    if visited.contains(&name) {
        return Some(t);
    }
    visited.insert(name.clone());

    let replacement = equations
        .iter()
        .find_map(|(t1, t2)| match (*t1 == t, *t2 == t) {
            (true, _) => Some(t2.clone()),
            (_, true) => Some(t1.clone()),
            _ => None,
        });

    match replacement {
        Some(new_type) if new_type != t => get_equation_internal(equations, new_type, visited),
        _ => replacement.or(Some(t)),
    }
}

fn pick_equation(equations: &Equations) -> Option<(Type, Type)> {
    equations.iter().last().cloned()
}

fn remove_equation(equations: Equations, (t1, t2): (Type, Type)) -> Equations {
    let mut equations = equations.clone();
    equations.remove(&(t1, t2));
    equations
}

pub fn unify(equations: Equations, substitutions: Equations) -> Result<Equations> {
    let Some((t1, t2)) = pick_equation(&equations) else {
        return Ok(substitutions);
    };
    let remaining = remove_equation(equations, (t1.clone(), t2.clone()));

    match (t1, t2) {
        // (EU{(p,p)},S) => (E,S)
        (t1, t2) if t1 == t2 => unify(remaining, substitutions),
        // (EU{(a,p)},S) => ([p/a]E,{(a,p)}U[p/a]S) ただしa∉FTV(p)
        (Type::Variable { name }, t2) => unify2(remaining, substitutions, name, t2),
        // (EU{(p,a)},S) => ([p/a]E,{(a,p)}U[p/a]S) ただしp∉FTV(a)
        (t1, Type::Variable { name }) => unify2(remaining, substitutions, name, t1),
        // (EU{(p1->r1,p2->r2)},S) => (EU{(p1,p2),(r1,r2)},S)
        (
            Type::Function {
                domain: domain1,
                range: range1,
            },
            Type::Function {
                domain: domain2,
                range: range2,
            },
        ) => {
            let new_equations = add_equation(remaining, *domain1, *domain2);
            let new_equations = add_equation(new_equations, *range1, *range2);
            unify(new_equations, substitutions)
        }
        // (EU{(List(t1),List(t2))},S) => (EU{(t1,t2)},S)
        (Type::List(t1), Type::List(t2)) => {
            let new_equations = add_equation(remaining, *t1, *t2);
            unify(new_equations, substitutions)
        }
        _ => bail!(UnificationError::Impossible),
    }
}

fn unify2(
    equations: Equations,
    substitutions: Equations,
    variable_name: Symbol,
    t: Type,
) -> Result<Equations> {
    if occurs_check(variable_name.clone(), t.clone()) {
        bail!(UnificationError::CircularReference);
    }

    let substituted_equations =
        apply_substitution_to_all(equations, variable_name.clone(), t.clone());
    let substituted_substitutions =
        apply_substitution_to_all(substitutions, variable_name.clone(), t.clone());

    let result_substitutions = add_equation(
        substituted_substitutions,
        Type::Variable {
            name: variable_name.clone(),
        },
        t.clone(),
    );
    unify(substituted_equations, result_substitutions)
}

fn occurs_check(variable_name: Symbol, t: Type) -> bool {
    match t {
        Type::Base(_) => false,
        Type::List(_) => false,
        Type::Variable { name } => variable_name == name,
        Type::Function { domain, range } => {
            occurs_check(variable_name.clone(), *domain)
                || occurs_check(variable_name.clone(), *range)
        }
    }
}

fn apply_substitution_to_all(equations: Equations, variable_name: Symbol, t: Type) -> Equations {
    equations
        .into_iter()
        .map(|(t1, t2)| {
            (
                t1.apply_substitution_for_type(variable_name.clone(), t.clone()),
                t2.apply_substitution_for_type(variable_name.clone(), t.clone()),
            )
        })
        .collect()
}
