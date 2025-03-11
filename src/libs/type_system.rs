pub mod type_environment;

use anyhow::Result;
use thiserror::Error;
use type_environment::TypeEnvironment;

use crate::syntax::ast::Expression;

#[derive(Debug, Error)]
enum TypeInferenceError {
    #[error("Invalid type: {0}")]
    Impossible(Expression),
}

pub fn type_inference(expression: Expression) -> Result<TypeEnvironment> {
    Ok(TypeEnvironment::default())
}
