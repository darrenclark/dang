fn main() {
    let code = "add(1, 2)";
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_dang::language())
        .expect("Error loading dang grammar");
    let tree = parser.parse(code, None).unwrap();
    println!("{:?}", tree);
}
