use anyhow::Result;
use thiserror::Error;

use crate::{syntax::ast::Expression, type_system::type_environment::TypeEnvironment};

#[derive(Debug, Error)]
enum UnificationError {
    #[error("Unification impossible {0}")]
    Impossible(Expression),
}

pub fn unify(type_environment: TypeEnvironment) -> Result<TypeEnvironment> {
    Ok(type_environment)
}
