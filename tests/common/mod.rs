use dang::interpreter::Exception;
use dang::interpreter::Interpreter;
use dang::value::Value;

pub fn run(code: &str) -> Result<Value, Exception> {
    let mut interpreter = Interpreter::new();
    let result = dang::dang_parser::parse(code, "(run)");
    if let Ok(pairs) = result {
        interpreter.eval(&pairs)
    } else {
        panic!("{}", result.unwrap_err());
    }
}
