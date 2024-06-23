use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dang.pest"]
pub struct DangParser;
