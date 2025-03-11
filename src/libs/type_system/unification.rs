use anyhow::Result;
use thiserror::Error;

use crate::type_system::type_environment::TypeEnvironment;

#[derive(Debug, Error)]
enum UnificationError {
    #[error("Unification impossible")]
    Impossible,
}

pub fn unify(type_environment: TypeEnvironment) -> Result<TypeEnvironment> {
    Ok(type_environment)
}
