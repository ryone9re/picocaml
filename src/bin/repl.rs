use anyhow::Result;
use picocaml::{
    analysis::{parser::parse_expression, tokenizer::tokenize},
    execution::{environment::Environment, evaluation::eval},
    type_system::{inference::type_inference, type_environment::TypeEnvironment},
};
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() -> Result<()> {
    let mut global_type_environment = TypeEnvironment::default();
    let mut global_environment = Environment::default();

    let mut rl = DefaultEditor::new()?;
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                match parse_expression(tokenize(line)) {
                    Ok(expression) => {
                        let infered =
                            type_inference(global_type_environment.clone(), expression.clone());
                        if let Err(e) = infered {
                            eprintln!("{}", e);
                            continue;
                        }
                        let (type_environment, ty) = infered.unwrap();
                        global_type_environment = type_environment;
                        println!("Type: {}", ty);

                        let evaluated = eval(global_environment.clone(), expression.clone());
                        if let Err(e) = evaluated {
                            eprintln!("{}", e);
                            continue;
                        }
                        let (environment, value) = evaluated.unwrap();
                        global_environment = environment;
                        println!("Value: {}", value);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Bye ;)");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
