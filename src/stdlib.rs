use include_dir::{include_dir, Dir, File};

use crate::{ast::Node, dang_parser};

static STDLIB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/stdlib");
#[allow(dead_code)]
static STDLIB_LAST_MODIFIED: &str = env!("DANG_STDLIB_LAST_MODIFIED");

pub fn load_stdlib() -> Vec<(String, Node)> {
    STDLIB_DIR.files().map(load_file).collect()
}

pub fn load_stdlib_file(path: &str) -> Option<&'static str> {
    STDLIB_DIR
        .get_file(path)
        .map(|f| f.contents_utf8().unwrap())
}

fn load_file(file: &File) -> (String, Node) {
    let module = file.path().to_str().unwrap().to_owned();
    match dang_parser::parse(file.contents_utf8().unwrap(), file.path().to_str().unwrap()) {
        Ok(node) => (module, node),
        Err(err) => {
            panic!("Failed to load standard library\n{}", err)
        }
    }
}
