use std::fs;

use clap::Parser;
use dang::compiler::{Compiler, Input};
use dang::dang_parser::Rule;
use dang::program::Program;
use dang::vm::disassembler::disassemble;
use dang::vm::VM;
use dirs::home_dir;
use pest::iterators::Pair;
use pretty::termcolor::{ColorChoice, StandardStream};
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

    /// print disassembly of byte code
    #[arg(long)]
    print_disasm: bool,

    /// dump debug info on crashes
    #[arg(long)]
    debug: bool,

    /// disable default std library imports
    #[arg(long)]
    no_std: bool,

    /// path to .dang file to execute
    path: Option<std::path::PathBuf>,

    /// output path for flamegraph (requires the flamegraph feature)
    #[arg(long)]
    flamegraph: Option<std::path::PathBuf>,

    /// args passed to the dang program
    #[arg(last(false))]
    args: Vec<String>,
}

const HISTORY_FILE: &str = ".dang_history";

fn main() -> Result<()> {
    let args = Cli::parse();

    if (args.print_pest_parse_output || args.print_ast || args.print_disasm) && args.path.is_none()
    {
        panic!("expected a file")
    }

    match &args.path {
        Some(path) => run_file(&args, path),
        None => repl(&args),
    }
}

fn run_file(args: &Cli, path: &std::path::PathBuf) -> Result<()> {
    let file = fs::read_to_string(path)?;

    let execute = !args.print_pest_parse_output && !args.print_ast && !args.print_disasm;

    if args.print_pest_parse_output {
        print_pest_parse_output(&file);
    }
    if args.print_ast {
        print_ast(&file, path.to_str().unwrap());
    }

    let mut compiler = Compiler::default();
    if args.no_std {
        compiler.implicit_imports.clear();
    }

    compiler
        .module_search_paths
        .push(path.parent().unwrap().to_owned());

    let mut program = Program::default();

    let input = Input::SourceCode {
        text: file,
        name: path.to_str().unwrap().to_owned(),
    };

    match compiler.compile(input, &mut program) {
        Ok(_) => {
            let module_name = program.module_names().last().cloned().unwrap();
            let module = program.get_module(&module_name).unwrap();
            if args.print_disasm {
                disassemble(&module.function);
            } else if execute {
                let mut vm = VM::new(module.function.clone());
                vm.set_argv(args.args.clone());
                match vm.run() {
                    Ok(value) => {
                        value
                            .to_doc()
                            .render_colored(80, StandardStream::stdout(ColorChoice::Auto))
                            .unwrap();
                        println!();

                        if cfg!(feature = "stats") {
                            vm.print_stats();
                        }
                        if let Some(f) = &args.flamegraph {
                            if cfg!(feature = "flamegraph") {
                                vm.write_flamegraph(f).expect("Failed to write flamegraph:");
                            } else {
                                println!("ERROR: flamegraph output requested but flamegraph feature is not enabled");
                            }
                        }
                    }
                    Err(e) => {
                        if args.debug {
                            vm.print_disasm();
                            vm.print_stack();
                        }

                        vm.print_stacktrace(e);
                    }
                }
            }
        }
        Err(errors) => {
            for e in errors {
                println!("{}", e);
            }
        }
    }

    Ok(())
}

fn repl(_: &Cli) -> Result<()> {
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
                let _ = rl.add_history_entry(line.as_str());
                println!("TODO: Implement REPL with VM (line {})", lineno);
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
