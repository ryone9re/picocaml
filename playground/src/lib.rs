use serde::Serialize;

use picocaml::{
    analysis::{parser::parse, tokenizer::tokenize},
    execution::{environment::Environment, evaluation::eval},
    type_system::{inference::infer, type_environment::TypeEnvironment},
};

#[derive(Debug, Serialize)]
pub struct LabReport {
    pub source: String,
    pub ast: Option<String>,
    pub ty: Option<String>,
    pub value: Option<String>,
    pub phase: Option<String>,
    pub error: Option<String>,
}

fn evaluate(source: &str) -> LabReport {
    let source = source.trim().to_owned();
    let expression = match parse(tokenize(source.clone())) {
        Ok(expression) => expression,
        Err(error) => return error_report(source, "parse", error.to_string(), None, None),
    };

    let ast = Some(format!("{expression:#?}"));
    let (_type_environment, ty) = match infer(TypeEnvironment::default(), expression.clone()) {
        Ok(result) => result,
        Err(error) => return error_report(source, "type", error.to_string(), ast, None),
    };

    let ty = ty.to_string();
    match eval(Environment::default(), expression) {
        Ok((_, value)) => LabReport {
            source,
            ast,
            ty: Some(ty),
            value: Some(value.to_string()),
            phase: None,
            error: None,
        },
        Err(error) => error_report(source, "evaluation", error.to_string(), ast, Some(ty)),
    }
}

fn error_report(
    source: String,
    phase: &str,
    error: String,
    ast: Option<String>,
    ty: Option<String>,
) -> LabReport {
    LabReport {
        source,
        ast,
        ty,
        value: None,
        phase: Some(phase.to_owned()),
        error: Some(error),
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run(source: String) -> String {
    serde_json::to_string(&evaluate(&source)).expect("LabReport should be serializable")
}
