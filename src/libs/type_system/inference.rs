use crate::{
    adapter::{Symbol, TypeTraverseHistory, unique_symbol},
    syntax::ast::Expression,
    type_system::{
        type_environment::TypeEnvironment,
        type_scheme::TypeScheme,
        types::{BaseType, Type},
    },
};
use anyhow::{Ok, Result, bail};
use thiserror::Error;

use super::types::free_type_variables;

type InferenceResult = Result<(TypeEnvironment, Type)>;

#[derive(Debug, Error)]
enum TypeInferenceError {
    #[error("Inference impossible: {0}")]
    Impossible(Expression),
    #[error("Invalid type: {0}")]
    InvalidType(Expression),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(Expression),
}

pub fn type_inference(
    type_environment: TypeEnvironment,
    expression: Expression,
) -> InferenceResult {
    let (inferred_environment, inferred_type) = infer(type_environment, expression)?;
    let unified_environment = inferred_environment.unify_equations()?;
    let normalized_type =
        unified_environment.normalize_type(TypeTraverseHistory::new(), inferred_type)?;

    Ok((unified_environment, normalized_type))
}

fn infer(type_environment: TypeEnvironment, expression: Expression) -> InferenceResult {
    match expression {
        Expression::Integer(_) => infer_integer(type_environment, expression),
        Expression::Bool(_) => infer_bool(type_environment, expression),
        Expression::Variable(_) => infer_variable(type_environment, expression),
        Expression::Plus {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::Minus {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::Times {
            expression1,
            expression2,
        } => infer_binary_operation(type_environment, *expression1, *expression2),
        Expression::LessThan {
            expression1,
            expression2,
        } => infer_binary_predicate(type_environment, *expression1, *expression2),
        Expression::If {
            predicate,
            consequent,
            alternative,
        } => infer_if(type_environment, *predicate, *consequent, *alternative),
        Expression::Let {
            variable,
            bound,
            body,
        } => infer_let(type_environment, variable, *bound, *body),
        Expression::Fun { parameter, body } => infer_fun(type_environment, parameter, *body),
        Expression::App { function, argument } => infer_app(type_environment, *function, *argument),
        Expression::LetRec {
            variable,
            bound_function,
            body,
        } => infer_let_rec(type_environment, variable, *bound_function, *body),
        Expression::Nil => infer_nil(type_environment),
        Expression::Cons { car, cdr } => infer_cons(type_environment, *car, *cdr),
        Expression::Match {
            scrutinee,
            nil_case,
            cons_pattern: (car, cdr, cons_case),
        } => infer_match(
            type_environment,
            *scrutinee,
            *nil_case,
            (car, cdr, *cons_case),
        ),
    }
}

fn infer_integer(type_environment: TypeEnvironment, expression: Expression) -> InferenceResult {
    match expression {
        Expression::Integer(_) => Ok((type_environment, Type::Base(BaseType::Integer))),
        _ => bail!(TypeInferenceError::Impossible(expression)),
    }
}

fn infer_bool(type_environment: TypeEnvironment, expression: Expression) -> InferenceResult {
    match expression {
        Expression::Bool(_) => Ok((type_environment, Type::Base(BaseType::Bool))),
        _ => bail!(TypeInferenceError::Impossible(expression)),
    }
}

fn infer_variable(type_environment: TypeEnvironment, expression: Expression) -> InferenceResult {
    match &expression {
        Expression::Variable(name) => {
            let variable_type = type_environment.get_variable_type(name)?;

            Ok((type_environment, variable_type))
        }
        _ => bail!(TypeInferenceError::UndefinedVariable(expression)),
    }
}

fn infer_binary_operation(
    type_environment: TypeEnvironment,
    expression1: Expression,
    expression2: Expression,
) -> InferenceResult {
    let (type_environment, expression1_type) = infer(type_environment, expression1)?;
    let (type_environment, expression2_type) = infer(type_environment, expression2)?;

    let type_environment =
        type_environment.add_equation(expression1_type.clone(), expression2_type.clone());

    Ok((type_environment, expression1_type))
}

fn infer_binary_predicate(
    type_environment: TypeEnvironment,
    expression1: Expression,
    expression2: Expression,
) -> InferenceResult {
    let (type_environment, expression1_type) = infer(type_environment, expression1)?;
    let (type_environment, expression2_type) = infer(type_environment, expression2)?;

    let type_environment =
        type_environment.add_equation(expression1_type.clone(), expression2_type.clone());

    Ok((type_environment, Type::Base(BaseType::Bool)))
}

fn infer_if(
    type_environment: TypeEnvironment,
    predicate: Expression,
    consequent: Expression,
    alternative: Expression,
) -> InferenceResult {
    let (type_environment, predicate_type) = infer(type_environment, predicate.clone())?;
    let type_environment =
        type_environment.add_equation(predicate_type.clone(), Type::Base(BaseType::Bool));

    let (type_environment, consequent_type) = infer(type_environment, consequent.clone())?;
    let (type_environment, alternative_type) = infer(type_environment, alternative.clone())?;

    let type_environment =
        type_environment.add_equation(consequent_type.clone(), alternative_type.clone());

    Ok((type_environment, consequent_type))
}

fn infer_let(
    type_environment: TypeEnvironment,
    variable: Symbol,
    bound: Expression,
    body: Expression,
) -> InferenceResult {
    let (type_environment, bound_type) = infer(type_environment, bound)?;

    let free_variables =
        type_environment.get_unbound_variables(free_type_variables(bound_type.clone()).into_iter());

    let type_environment = type_environment.substitute_variable(
        variable.clone(),
        TypeScheme::new_polymorphic_type_scheme(free_variables.into_iter(), bound_type),
    )?;

    infer(type_environment, body)
}

fn infer_fun(
    type_environment: TypeEnvironment,
    parameter: Symbol,
    body: Expression,
) -> InferenceResult {
    let unique_parameter = unique_symbol();

    let parameter_type = Type::Variable {
        name: unique_parameter.clone(),
    };

    let type_environment = type_environment.substitute_variable(
        parameter.clone(),
        TypeScheme::new_monomorphic_type_scheme(parameter_type.clone()),
    )?;

    let (type_environment, body_type) = infer(type_environment, body)?;
    let substitued_body_type = body_type.apply_substitution(parameter, unique_parameter);

    Ok((
        type_environment,
        Type::Function {
            domain: parameter_type.clone().into(),
            range: substitued_body_type.into(),
        },
    ))
}

fn infer_app(
    type_environment: TypeEnvironment,
    function: Expression,
    argument: Expression,
) -> InferenceResult {
    let (type_environment, function_type) = infer(type_environment, function.clone())?;
    let Type::Function { domain, range } = function_type else {
        bail!(TypeInferenceError::InvalidType(function));
    };

    let (type_environment, argument_type) = infer(type_environment, argument.clone())?;
    let type_environment = type_environment.add_equation(*domain, argument_type);

    Ok((type_environment, *range))
}

fn infer_let_rec(
    type_environment: TypeEnvironment,
    variable: Symbol,
    bound_function: Expression,
    body: Expression,
) -> InferenceResult {
    // 1. 仮の関数型を作成
    let recursive_function_argument_type = Type::Variable {
        name: unique_symbol(),
    };
    let recursive_function_return_type = Type::Variable {
        name: unique_symbol(),
    };
    let recursice_function_type = Type::Function {
        domain: recursive_function_argument_type.clone().into(),
        range: recursive_function_return_type.clone().into(),
    };

    // 2. 単相的な型として関数を型環境に追加（関数本体の型推論用）
    let temporal_environment = type_environment.substitute_variable(
        variable.clone(),
        TypeScheme::new_monomorphic_type_scheme(recursice_function_type.clone()),
    )?;

    // 3. 関数本体の型推論
    let (bound_function_environment, bound_function_type) =
        infer(temporal_environment, bound_function.clone())?;
    let Type::Function { domain, range } = bound_function_type else {
        bail!(TypeInferenceError::InvalidType(bound_function));
    };

    // 4. 関数型の制約を追加
    let type_environment = bound_function_environment
        .add_equation(recursive_function_argument_type.clone(), *domain)
        .add_equation(recursive_function_return_type.clone(), *range);

    // 5. 単一化して最終的な関数型を得る
    let unified_environment = type_environment.clone().unify_equations()?;
    let actual_function_type = unified_environment
        .normalize_type(TypeTraverseHistory::new(), recursice_function_type.clone())?;

    // 6. 自由型変数を抽出し、多相型化
    let free_variables = type_environment
        .get_unbound_variables(free_type_variables(actual_function_type.clone()).into_iter());

    // 7. 多相型として関数を型環境に追加し、本体の型推論
    let type_environment = type_environment.substitute_variable(
        variable.clone(),
        TypeScheme::new_polymorphic_type_scheme(free_variables.into_iter(), actual_function_type),
    )?;

    infer(type_environment, body)
}

fn infer_nil(type_environment: TypeEnvironment) -> InferenceResult {
    Ok((
        type_environment,
        Type::List(
            Type::Variable {
                name: unique_symbol(),
            }
            .into(),
        ),
    ))
}

fn infer_cons(
    type_environment: TypeEnvironment,
    car: Expression,
    cdr: Expression,
) -> InferenceResult {
    let (type_environment, car_type) = infer(type_environment, car)?;

    let (type_environment, cdr_type) = infer(type_environment, cdr.clone())?;
    let Type::List(element_type) = cdr_type.clone() else {
        bail!(TypeInferenceError::InvalidType(cdr));
    };

    let type_environment = type_environment.add_equation(car_type, *element_type);

    Ok((type_environment, cdr_type))
}

fn infer_match(
    type_environment: TypeEnvironment,
    scrutinee: Expression,
    nil_case: Expression,
    (car, cdr, cons_case): (Symbol, Symbol, Expression),
) -> InferenceResult {
    let (type_environment, scrutinee_type) = infer(type_environment, scrutinee.clone())?;
    let (type_environment, element_type) = match scrutinee_type {
        Type::List(element_type) => (type_environment, *element_type),
        variable @ Type::Variable { .. } => {
            let element_type = Type::Variable {
                name: unique_symbol(),
            };
            let type_environment =
                type_environment.add_equation(variable, Type::List(element_type.clone().into()));
            (type_environment, element_type)
        }
        _ => bail!(TypeInferenceError::InvalidType(scrutinee.clone())),
    };

    let (type_environment, nil_case_type) = infer(type_environment, nil_case)?;

    let type_environment = type_environment
        .substitute_variable(
            car.clone(),
            TypeScheme::new_monomorphic_type_scheme(element_type.clone()),
        )?
        .substitute_variable(
            cdr.clone(),
            TypeScheme::new_monomorphic_type_scheme(Type::List(element_type.into())),
        )?;
    let (type_environment, cons_case_type) = infer(type_environment, cons_case)?;
    let type_environment =
        type_environment.add_equation(nil_case_type.clone(), cons_case_type.clone());

    Ok((type_environment, nil_case_type))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_infer_integer() {
        let expression = Expression::Integer(10);

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_bool() {
        let expression = Expression::Bool(true);

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_variable() {
        let env = TypeEnvironment::default()
            .substitute_variable(
                "x".to_string(),
                TypeScheme::new_monomorphic_type_scheme(Type::Base(BaseType::Integer)),
            )
            .unwrap();

        let expression = Expression::Variable("x".to_string());

        let result = infer(env, expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_undefined_variable() {
        let expression = Expression::Variable("unknown".to_string());

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_plus() {
        let expression = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_minus() {
        let expression = Expression::Minus {
            expression1: Expression::Integer(10).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_times() {
        let expression = Expression::Times {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_less_than() {
        let expression = Expression::LessThan {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Integer(5).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_invalid_operation() {
        let expression = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Bool(true).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_if() {
        let expression = Expression::If {
            predicate: Expression::Bool(true).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Integer(2).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_if_with_invalid_predicate() {
        let expression = Expression::If {
            predicate: Expression::Integer(1).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Integer(2).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_if_with_mismatched_branches() {
        let expression = Expression::If {
            predicate: Expression::Bool(true).into(),
            consequent: Expression::Integer(1).into(),
            alternative: Expression::Bool(false).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_let() {
        let expression = Expression::Let {
            variable: "x".to_string(),
            bound: Expression::Integer(5).into(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(3).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_let_with_complex_bound() {
        let expression = Expression::Let {
            variable: "result".to_string(),
            bound: Expression::If {
                predicate: Expression::Bool(true).into(),
                consequent: Expression::Integer(10).into(),
                alternative: Expression::Integer(20).into(),
            }
            .into(),
            body: Expression::Variable("result".to_string()).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_fun() {
        let expression = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();

        match t {
            Type::Function { domain, range } => {
                assert_eq!(*domain, Type::Base(BaseType::Integer));
                assert_eq!(*range, Type::Base(BaseType::Integer));
            }
            _ => panic!("Expected function type, got: {:#?}", t),
        }
    }

    #[test]
    fn test_infer_identity_fun() {
        let expression = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Variable("x".to_string()).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();

        match t {
            Type::Function { domain, range } => {
                assert_eq!(*domain, *range);
            }
            _ => panic!("Expected function type, got: {:#?}", t),
        }
    }

    #[test]
    fn test_infer_app() {
        let function_expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let arg_expr = Expression::Integer(5);

        let expression = Expression::App {
            function: function_expr.into(),
            argument: arg_expr.into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_app_with_invalid_argument() {
        let function_expr = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let arg_expr = Expression::Bool(true);

        let expression = Expression::App {
            function: function_expr.into(),
            argument: arg_expr.into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_let_rec() {
        let expression = Expression::LetRec {
            variable: "fact".to_string(),
            bound_function: Expression::Fun {
                parameter: "n".to_string(),
                body: Expression::If {
                    predicate: Expression::LessThan {
                        expression1: Expression::Variable("n".to_string()).into(),
                        expression2: Expression::Integer(1).into(),
                    }
                    .into(),
                    consequent: Expression::Integer(1).into(),
                    alternative: Expression::Times {
                        expression1: Expression::Variable("n".to_string()).into(),
                        expression2: Expression::App {
                            function: Expression::Variable("fact".to_string()).into(),
                            argument: Expression::Minus {
                                expression1: Expression::Variable("n".to_string()).into(),
                                expression2: Expression::Integer(1).into(),
                            }
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            body: Expression::App {
                function: Expression::Variable("fact".to_string()).into(),
                argument: Expression::Integer(5).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_nil() {
        let expression = Expression::Nil;

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert!(matches!(t, Type::List(_)));
    }

    #[test]
    fn test_infer_cons() {
        let expression = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Nil.into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert!(
            matches!(t, Type::List(element_type) if matches!(*element_type, Type::Base(BaseType::Integer)))
        );
    }

    #[test]
    fn test_infer_cons_with_invalid_elements() {
        let expression = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Cons {
                car: Expression::Bool(true).into(),
                cdr: Expression::Nil.into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_match_nil_case() {
        let expression = Expression::Match {
            scrutinee: Expression::Nil.into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_match_cons_case() {
        let expression = Expression::Match {
            scrutinee: Expression::Cons {
                car: Expression::Integer(1).into(),
                cdr: Expression::Nil.into(),
            }
            .into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_match_with_invalid_scrutinee() {
        let expression = Expression::Match {
            scrutinee: Expression::Integer(5).into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Plus {
                    expression1: Expression::Variable("head".to_string()).into(),
                    expression2: Expression::Integer(1).into(),
                }
                .into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_match_with_mismatched_cases() {
        let expression = Expression::Match {
            scrutinee: Expression::Nil.into(),
            nil_case: Expression::Integer(0).into(),
            cons_pattern: (
                "head".to_string(),
                "tail".to_string(),
                Expression::Bool(true).into(),
            ),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_err());
    }

    #[test]
    fn test_infer_complex_arithmetic() {
        let expression = Expression::Plus {
            expression1: Expression::Integer(3).into(),
            expression2: Expression::Times {
                expression1: Expression::Integer(5).into(),
                expression2: Expression::Integer(2).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_complex_predicate() {
        let expression = Expression::LessThan {
            expression1: Expression::Plus {
                expression1: Expression::Integer(3).into(),
                expression2: Expression::Integer(5).into(),
            }
            .into(),
            expression2: Expression::Integer(10).into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Bool));
    }

    #[test]
    fn test_infer_complex_function() {
        let expression = Expression::Let {
            variable: "add".to_string(),
            bound: Expression::Fun {
                parameter: "x".to_string(),
                body: Expression::Fun {
                    parameter: "y".to_string(),
                    body: Expression::Plus {
                        expression1: Expression::Variable("x".to_string()).into(),
                        expression2: Expression::Variable("y".to_string()).into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
            body: Expression::App {
                function: Expression::App {
                    function: Expression::Variable("add".to_string()).into(),
                    argument: Expression::Integer(3).into(),
                }
                .into(),
                argument: Expression::Integer(4).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_list_sum() {
        let sum_function = Expression::Fun {
            parameter: "l".to_string(),
            body: Expression::Match {
                scrutinee: Expression::Variable("l".to_string()).into(),
                nil_case: Expression::Integer(0).into(),
                cons_pattern: (
                    "h".to_string(),
                    "t".to_string(),
                    Expression::Plus {
                        expression1: Expression::Variable("h".to_string()).into(),
                        expression2: Expression::App {
                            function: Expression::Variable("sum".to_string()).into(),
                            argument: Expression::Variable("t".to_string()).into(),
                        }
                        .into(),
                    }
                    .into(),
                ),
            }
            .into(),
        };

        let list_expr = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Cons {
                car: Expression::Integer(2).into(),
                cdr: Expression::Cons {
                    car: Expression::Integer(3).into(),
                    cdr: Expression::Nil.into(),
                }
                .into(),
            }
            .into(),
        };

        let expression = Expression::LetRec {
            variable: "sum".to_string(),
            bound_function: sum_function.into(),
            body: Expression::App {
                function: Expression::Variable("sum".to_string()).into(),
                argument: list_expr.into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_polymorphic_identity() {
        let id_function = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Variable("x".to_string()).into(),
        };

        let expression = Expression::Let {
            variable: "id".to_string(),
            bound: id_function.into(),
            body: Expression::If {
                predicate: Expression::App {
                    function: Expression::Variable("id".to_string()).into(),
                    argument: Expression::Bool(true).into(),
                }
                .into(),
                consequent: Expression::App {
                    function: Expression::Variable("id".to_string()).into(),
                    argument: Expression::Integer(42).into(),
                }
                .into(),
                alternative: Expression::Integer(0).into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }

    #[test]
    fn test_infer_polymorphic_map() {
        let map_function = Expression::Fun {
            parameter: "f".to_string(),
            body: Expression::Fun {
                parameter: "xs".to_string(),
                body: Expression::Match {
                    scrutinee: Expression::Variable("xs".to_string()).into(),
                    nil_case: Expression::Nil.into(),
                    cons_pattern: (
                        "h".to_string(),
                        "t".to_string(),
                        Expression::Cons {
                            car: Expression::App {
                                function: Expression::Variable("f".to_string()).into(),
                                argument: Expression::Variable("h".to_string()).into(),
                            }
                            .into(),
                            cdr: Expression::App {
                                function: Expression::App {
                                    function: Expression::Variable("map".to_string()).into(),
                                    argument: Expression::Variable("f".to_string()).into(),
                                }
                                .into(),
                                argument: Expression::Variable("t".to_string()).into(),
                            }
                            .into(),
                        }
                        .into(),
                    ),
                }
                .into(),
            }
            .into(),
        };

        let int_list = Expression::Cons {
            car: Expression::Integer(1).into(),
            cdr: Expression::Cons {
                car: Expression::Integer(2).into(),
                cdr: Expression::Cons {
                    car: Expression::Integer(3).into(),
                    cdr: Expression::Nil.into(),
                }
                .into(),
            }
            .into(),
        };

        let add_one = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::Plus {
                expression1: Expression::Variable("x".to_string()).into(),
                expression2: Expression::Integer(1).into(),
            }
            .into(),
        };

        let positive = Expression::Fun {
            parameter: "x".to_string(),
            body: Expression::LessThan {
                expression1: Expression::Integer(0).into(),
                expression2: Expression::Variable("x".to_string()).into(),
            }
            .into(),
        };

        let expression = Expression::LetRec {
            variable: "map".to_string(),
            bound_function: map_function.into(),
            body: Expression::Let {
                variable: "int_list".to_string(),
                bound: int_list.into(),
                body: Expression::Let {
                    variable: "incremented".to_string(),
                    bound: Expression::App {
                        function: Expression::App {
                            function: Expression::Variable("map".to_string()).into(),
                            argument: add_one.into(),
                        }
                        .into(),
                        argument: Expression::Variable("int_list".to_string()).into(),
                    }
                    .into(),
                    body: Expression::App {
                        function: Expression::App {
                            function: Expression::Variable("map".to_string()).into(),
                            argument: positive.into(),
                        }
                        .into(),
                        argument: Expression::Variable("incremented".to_string()).into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert!(
            matches!(t, Type::List(element_type) if matches!(*element_type, Type::Base(BaseType::Bool)))
        );
    }

    #[test]
    fn test_polymorphic_compose() {
        let compose_function = Expression::Fun {
            parameter: "f".to_string(),
            body: Expression::Fun {
                parameter: "g".to_string(),
                body: Expression::Fun {
                    parameter: "x".to_string(),
                    body: Expression::App {
                        function: Expression::Variable("f".to_string()).into(),
                        argument: Expression::App {
                            function: Expression::Variable("g".to_string()).into(),
                            argument: Expression::Variable("x".to_string()).into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
        };

        let multiply_by_10 = Expression::Fun {
            parameter: "n".to_string(),
            body: Expression::Times {
                expression1: Expression::Variable("n".to_string()).into(),
                expression2: Expression::Integer(10).into(),
            }
            .into(),
        };

        let double = Expression::Fun {
            parameter: "n".to_string(),
            body: Expression::Times {
                expression1: Expression::Variable("n".to_string()).into(),
                expression2: Expression::Integer(2).into(),
            }
            .into(),
        };

        let bool_to_int = Expression::Fun {
            parameter: "b".to_string(),
            body: Expression::If {
                predicate: Expression::Variable("b".to_string()).into(),
                consequent: Expression::Integer(1).into(),
                alternative: Expression::Integer(0).into(),
            }
            .into(),
        };

        let expression = Expression::Let {
            variable: "compose".to_string(),
            bound: compose_function.into(),
            body: Expression::Let {
                variable: "multiply_by_20".to_string(),
                bound: Expression::App {
                    function: Expression::App {
                        function: Expression::Variable("compose".to_string()).into(),
                        argument: multiply_by_10.into(),
                    }
                    .into(),
                    argument: double.into(),
                }
                .into(),
                body: Expression::Let {
                    variable: "int_of_bool".to_string(),
                    bound: bool_to_int.into(),
                    body: Expression::Let {
                        variable: "bool_to_int_multiplied".to_string(),
                        bound: Expression::App {
                            function: Expression::App {
                                function: Expression::Variable("compose".to_string()).into(),
                                argument: Expression::Variable("multiply_by_20".to_string()).into(),
                            }
                            .into(),
                            argument: Expression::Variable("int_of_bool".to_string()).into(),
                        }
                        .into(),
                        body: Expression::App {
                            function: Expression::Variable("bool_to_int_multiplied".to_string())
                                .into(),
                            argument: Expression::Bool(true).into(),
                        }
                        .into(),
                    }
                    .into(),
                }
                .into(),
            }
            .into(),
        };

        let result = type_inference(TypeEnvironment::default(), expression);

        assert!(result.is_ok());
        let (_, t) = result.unwrap();
        assert_eq!(t, Type::Base(BaseType::Integer));
    }
}
