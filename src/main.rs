mod ast;
mod dang_parser;
mod interpreter;

use std::fs;

use clap::Parser;
use dirs::home_dir;
use interpreter::Interpreter;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

#[derive(Parser)]
struct Cli {
    path: Option<std::path::PathBuf>,
}

const HISTORY_FILE: &str = ".dang_history";

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.path {
        Some(path) => run_file(&path),
        None => repl(),
    }
}

fn run_file(path: &std::path::PathBuf) -> Result<()> {
    let mut interpreter = Interpreter::new();
    let file = fs::read_to_string(path)?;
    handle_input(&mut interpreter, file, false);
    Ok(())
}

fn repl() -> Result<()> {
    let mut interpreter = Interpreter::new_repl();

    let history_file = home_dir().map(|p| {
        let mut path = p.clone();
        path.push(HISTORY_FILE);
        String::from(path.to_str().unwrap())
    });

    let mut rl = DefaultEditor::new()?;

    if let Some(f) = &history_file {
        let _ = rl.load_history(&f);
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                handle_input(&mut interpreter, line, true);
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    if let Some(f) = &history_file {
        rl.save_history(&f)?;
    }
    Ok(())
}

fn handle_input(interpreter: &mut Interpreter, line: String, print_result: bool) {
    /*let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_dang::language())
        .expect("Error loading dang grammar");
    let tree = parser.parse(line, None).unwrap();
    println!("{}", tree.root_node().to_sexp());*/

    let result = dang_parser::parse(&line);
    if let Ok(pairs) = result {
        match interpreter.eval(&pairs) {
            Ok(value) => {
                if print_result {
                    println!("{:?}", value)
                }
            }
            Err(exception) => println!("{}", exception),
        }
    } else {
        println!("{}", result.unwrap_err());
    }
}

/*fn print_tree(pair: Pair<Rule>, depth: usize) {
    if pair.as_rule() == Rule::EOI {
        return;
    }

    println!(
        "{:indent$}{:?}:  {}",
        "",
        pair.as_rule(),
        pair.as_str(),
        indent = depth * 2
    );
    pair.into_inner().for_each(|p| print_tree(p, depth + 1))
}*/
