#include "interpreter.h"
#include "lexer.h"
#include "parser.h"

int main() {
  Lexer lexer(" let x = 10; x = x + 2; return (x + 5) * 2;  ");
  auto tokens = lexer.lex();

  Parser parser(tokens);
  auto ast = parser.parse();

  Interpreter interpreter(ast);

  int result = interpreter.run();

  std::cout << result << std::endl;
}
