#include "lexer.h"

int main() {
  Lexer lexer(" 123  ");

  auto tokens = lexer.lex();

  std::cout << tokens.size() << std::endl;
}
