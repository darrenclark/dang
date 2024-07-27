use std::fs;

use clap::Parser;
use dang::ast::ImportKind;
use dang::compiler::Input;
use dang::dang_parser::Rule;
use dang::interpreter::Interpreter;
use dirs::home_dir;
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
    let file = fs::read_to_string(path)?;

    let execute = !args.print_pest_parse_output && !args.print_ast;

    if args.print_pest_parse_output {
        print_pest_parse_output(&file);
    }
    if args.print_ast {
        print_ast(&file, path.to_str().unwrap());
    }

    if execute {
        let mut interpreter = Interpreter::new();

        interpreter
            .compiler
            .module_search_paths
            .push(path.parent().unwrap().to_owned());

        interpreter.set_argv(args.args.clone());

        handle_input(&mut interpreter, file, path.to_str().unwrap(), false);
    }
    Ok(())
}

fn repl(args: &Cli) -> Result<()> {
    let mut interpreter = Interpreter::new_repl();
    // preload Std to ensure there aren't any build issues with it
    if let Err(e) = interpreter.run(Input::ModuleName("Std".to_owned())) {
        panic!("Failed to load Std: {:?}", e);
    }
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
    let mut lineno = 0;
    loop {
        lineno += 1;
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let module_name = format!("(repl:{})", lineno);
                let _ = rl.add_history_entry(line.as_str());
                if handle_input(&mut interpreter, line, &module_name, true) {
                    let import = ImportKind::AllFields { module_name };
                    interpreter.compiler.implicit_imports.insert(0, import);
                }
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

fn handle_input(
    interpreter: &mut Interpreter,
    line: String,
    source: &str,
    print_result: bool,
) -> bool {
    match interpreter.run(Input::SourceCode {
        text: line,
        name: source.to_owned(),
    }) {
        Ok(value) => {
            if print_result {
                println!("{:?}", value)
            }
            true
        }
        Err(exception) => {
            println!("{}", exception);
            false
        }
    }
}

fn print_pest_parse_output(input: &str) {
    let result = dang::dang_parser::parse_pest_only(input);
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
    let result = dang::dang_parser::parse(input, source);
    if let Ok(ast) = result {
        println!("{:#?}", ast);
    } else {
        println!("{}", result.unwrap_err());
    }
}
