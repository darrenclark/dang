#include "interpreter.h"
#include "lexer.h"
#include "parser.h"

int main() {
  Lexer lexer(" return 80 + 5 * 2;  ");
  auto tokens = lexer.lex();

  Parser parser(tokens);
  auto ast = parser.parse();

  Interpreter interpreter(ast);

  int result = interpreter.run();

  std::cout << result << std::endl;
}
