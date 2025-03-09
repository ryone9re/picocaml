use std::{
    collections::HashMap,
    ops::{Add, Mul, Sub},
};

use anyhow::{Result, anyhow, bail};
use thiserror::Error;

use crate::{
    ast::Expression,
    adapter::{RBool, RInteger, Variable},
    value::Value,
};

#[derive(Debug, Error)]
enum EvalError {
    #[error("Invalid expression")]
    InvalidExpression,
    #[error("Undefined variable: {0}")]
    UndefinedVariable(Variable),
    #[error("Type error!")]
    TypeError,
}

pub type Environment = HashMap<Variable, Box<Value>>;

pub type TypeEnvironment = HashMap<Variable, ()>;

#[derive(Debug, Default, Clone)]
pub struct Structure {
    environment: Environment,
    type_environment: TypeEnvironment,
}

impl Structure {
    fn assign_variable(self, variable: Variable, value: Box<Value>) -> Result<Structure> {
        let mut new_structure = self.clone();
        new_structure.environment.insert(variable, value);
        Ok(new_structure)
    }
}

pub fn eval(
    structure: Structure,
    expression: Box<Expression>,
) -> anyhow::Result<(Structure, Box<Value>)> {
    let (new_structure, value) = match *expression {
        Expression::Integer(n) => eval_integer(structure, n)?,
        Expression::Bool(b) => eval_bool(structure, b)?,
        Expression::Variable(variable) => eval_variable(structure, variable)?,
        Expression::Plus { e1, e2 } => eval_plus(structure, e1, e2)?,
        Expression::Minus { e1, e2 } => eval_minus(structure, e1, e2)?,
        Expression::Times { e1, e2 } => eval_times(structure, e1, e2)?,
        Expression::LessThan { e1, e2 } => eval_lt(structure, e1, e2)?,
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => eval_if(structure, predicate, consequent, alternative)?,
        Expression::Let {
            variable,
            bound,
            body,
        } => eval_let(structure, variable, bound, body)?,
        Expression::Fun { variable, body } => eval_fun(structure, variable, body)?,
        Expression::App { function, argument } => todo!(),
        Expression::LetRec {
            variable,
            bound_function,
            body,
        } => todo!(),
        Expression::Nil => eval_nil(structure)?,
        Expression::Cons { car, cdr } => eval_cons(structure, car, cdr)?,
        Expression::Match {
            scrutinee,
            nil_case,
            cons_case,
        } => todo!(),
    };

    Ok((new_structure, value))
}

fn eval_integer(structure: Structure, n: RInteger) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Integer(n).into()))
}

fn eval_bool(structure: Structure, b: RBool) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Bool(b).into()))
}

fn eval_variable(structure: Structure, variable: Variable) -> Result<(Structure, Box<Value>)> {
    let value = structure
        .environment
        .get(&variable)
        .ok_or(anyhow!(EvalError::UndefinedVariable(variable.clone())))?
        .clone();
    Ok((structure, value))
}

fn eval_plus(
    structure: Structure,
    e1: Box<Expression>,
    e2: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.add(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_minus(
    structure: Structure,
    e1: Box<Expression>,
    e2: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.sub(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_times(
    structure: Structure,
    e1: Box<Expression>,
    e2: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Integer(e1_value.mul(e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_lt(
    structure: Structure,
    e1: Box<Expression>,
    e2: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, e1) = eval(structure.clone(), e1)?;
    let (_, e2) = eval(structure.clone(), e2)?;

    if let (Value::Integer(e1_value), Value::Integer(e2_value)) = (*e1, *e2) {
        return Ok((structure, Value::Bool(e1_value.lt(&e2_value)).into()));
    }

    bail!(EvalError::InvalidExpression)
}

fn eval_if(
    structure: Structure,
    predicate: Box<Expression>,
    consequent: Box<Expression>,
    alternative: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, predicate) = eval(structure.clone(), predicate)?;

    match *predicate {
        Value::Bool(b) if b => eval(structure, consequent),
        Value::Bool(b) if !b => eval(structure, alternative),
        _ => bail!(EvalError::TypeError),
    }
}

fn eval_let(
    structure: Structure,
    variable: Variable,
    bound: Box<Expression>,
    body: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, bound) = eval(structure.clone(), bound)?;
    let new_structure = structure.assign_variable(variable, bound)?;
    eval(new_structure, body)
}

fn eval_fun(
    structure: Structure,
    variable: Variable,
    body: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let captured_environment = structure.environment.clone();

    Ok((
        structure,
        Value::Closure {
            variable,
            body,
            environment: captured_environment,
        }
        .into(),
    ))
}

// fn eval_app(structure: Structure)

fn eval_nil(structure: Structure) -> Result<(Structure, Box<Value>)> {
    Ok((structure, Value::Nil.into()))
}

fn eval_cons(
    structure: Structure,
    car: Box<Expression>,
    cdr: Box<Expression>,
) -> Result<(Structure, Box<Value>)> {
    let (_, car) = eval(structure.clone(), car)?;
    let (_, cdr) = eval(structure.clone(), cdr)?;

    Ok((structure, Value::Cons { car, cdr }.into()))
}
