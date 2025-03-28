use std::collections::HashSet;

use crate::{
    adapter::{Symbol, unique_symbol},
    type_system::types::Type,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    variables: HashSet<Symbol>,
    base_type: Type,
}

impl TypeScheme {
    pub fn new_monomorphic_type_scheme(t: Type) -> Self {
        Self {
            variables: HashSet::new(),
            base_type: t,
        }
    }

    pub fn new_polymorphic_type_scheme<T: Iterator<Item = Symbol>>(
        variable_names: T,
        base_type: Type,
    ) -> Self {
        Self {
            variables: HashSet::from_iter(variable_names),
            base_type,
        }
    }

    pub fn instantiate(self) -> Type {
        let variables = self.variables.clone();
        let mut base_type = self.base_type;

        variables.into_iter().for_each(|variable| {
            base_type = base_type.clone().apply_substitution_for_type(
                variable,
                Type::Variable {
                    name: unique_symbol(),
                },
            )
        });

        base_type
    }
}
