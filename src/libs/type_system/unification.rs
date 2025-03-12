use anyhow::Result;
use thiserror::Error;

use crate::type_system::type_environment::TypeEnvironment;

use super::type_environment::Type;

#[derive(Debug, Error)]
enum UnificationError {
    #[error("Unification impossible")]
    Impossible,
}

pub fn unify(type_environment: TypeEnvironment) -> Result<TypeEnvironment> {
    Ok(type_environment)
}

pub fn normalize(type_environment: TypeEnvironment, inferred_type: Type) -> Result<Type> {
    Ok(inferred_type)
}
