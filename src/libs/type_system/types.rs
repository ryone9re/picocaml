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
