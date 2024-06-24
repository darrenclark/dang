mod ast;
mod dang_parser;
mod interpreter;

use dirs::home_dir;
use interpreter::interpret;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

const HISTORY_FILE: &str = ".dang_history";

fn main() -> Result<()> {
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
                handle_input(line);
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

fn handle_input(line: String) {
    /*let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_dang::language())
        .expect("Error loading dang grammar");
    let tree = parser.parse(line, None).unwrap();
    println!("{}", tree.root_node().to_sexp());*/

    let result = dang_parser::parse(&line);
    if let Ok(pairs) = result {
        let val = interpret(pairs);
        println!("{:?}", val);
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
