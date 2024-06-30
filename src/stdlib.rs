use include_dir::{include_dir, Dir, File};

use crate::{ast::Node, dang_parser};

static STDLIB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/stdlib");

pub fn load_stdlib() -> Vec<Node> {
    STDLIB_DIR.files().map(load_file).collect()
}

fn load_file(file: &File) -> Node {
    match dang_parser::parse(file.contents_utf8().unwrap(), file.path().to_str().unwrap()) {
        Ok(node) => node,
        Err(err) => {
            panic!("Failed to load standard library\n{}", err)
        }
    }
}
