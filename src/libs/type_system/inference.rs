use crate::type_system::type_environment::TypeEnvironment;
use anyhow::Result;
use thiserror::Error;

use crate::syntax::ast::Expression;

#[derive(Debug, Error)]
enum TypeInferenceError {
    #[error("Invalid type: {0}")]
    Impossible(Expression),
}

pub fn type_inference(expression: Expression) -> Result<TypeEnvironment> {
    Ok(TypeEnvironment::default())
}
