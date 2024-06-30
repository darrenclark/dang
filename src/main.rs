mod ast;
mod dang_parser;
mod interpreter;
mod native_funcs;
mod stdlib;
mod string_utils;

use std::fs;

use clap::Parser;
use dang_parser::Rule;
use dirs::home_dir;
use interpreter::Interpreter;
use pest::iterators::Pair;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

/// dang programming language
#[derive(Parser)]
struct Cli {
    /// debug: print pest parse output (instead of executing)
    #[arg(long)]
    print_pest_parse_output: bool,

    /// debug: print ast (instead of executing)
    #[arg(long)]
    print_ast: bool,

    /// path to .dang file to execute
    path: Option<std::path::PathBuf>,

    /// args passed to the dang program
    #[arg(last(false))]
    args: Vec<String>,
}

const HISTORY_FILE: &str = ".dang_history";

fn main() -> Result<()> {
    let args = Cli::parse();

    if (args.print_pest_parse_output || args.print_ast) && args.path.is_none() {
        panic!("expected a file")
    }

    match &args.path {
        Some(path) => run_file(&args, path),
        None => repl(&args),
    }
}

fn run_file(args: &Cli, path: &std::path::PathBuf) -> Result<()> {
    let mut interpreter = Interpreter::new();
    interpreter.set_argv(args.args.clone());

    let file = fs::read_to_string(path)?;

    let execute = !args.print_pest_parse_output && !args.print_ast;

    if args.print_pest_parse_output {
        print_pest_parse_output(&file);
    }
    if args.print_ast {
        print_ast(&file, path.to_str().unwrap());
    }

    if execute {
        handle_input(&mut interpreter, file, path.to_str().unwrap(), false);
    }
    Ok(())
}

fn repl(args: &Cli) -> Result<()> {
    let mut interpreter = Interpreter::new_repl();
    interpreter.set_argv(args.args.clone());

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
                handle_input(&mut interpreter, line, "(repl)", true);
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

fn handle_input(interpreter: &mut Interpreter, line: String, source: &str, print_result: bool) {
    /*let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_dang::language())
        .expect("Error loading dang grammar");
    let tree = parser.parse(line, None).unwrap();
    println!("{}", tree.root_node().to_sexp());*/

    let result = dang_parser::parse(&line, source);
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

fn print_pest_parse_output(input: &str) {
    let result = dang_parser::parse_pest_only(input);
    match result {
        Ok(pairs) => {
            for p in pairs {
                print_tree(p, 0);
            }
        }
        Err(err) => println!("{}", err),
    }
}

fn print_tree(pair: Pair<Rule>, depth: usize) {
    if pair.as_rule() == Rule::EOI {
        return;
    }

    let s = pair.as_str().lines().next().unwrap_or("");

    let t = pair
        .as_node_tag()
        .map(|t| format!("[{}] ", t))
        .unwrap_or("".to_owned());

    println!(
        "{:indent$}{}{:?}:  {}",
        "",
        t,
        pair.as_rule(),
        s,
        indent = depth * 2
    );
    pair.into_inner().for_each(|p| print_tree(p, depth + 1))
}

fn print_ast(input: &str, source: &str) {
    let result = dang_parser::parse(input, source);
    if let Ok(ast) = result {
        println!("{:#?}", ast);
    } else {
        println!("{}", result.unwrap_err());
    }
}
