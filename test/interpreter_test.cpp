#include <catch2/catch_test_macros.hpp>

#include "../src/interpreter.h"
#include "../src/lexer.h"
#include "../src/parser.h"

static ASTNodeProgram ast(std::string input) {
  Lexer l(input);
  Parser p(l.lex());
  return p.parse();
}

TEST_CASE("basic program can be run", "[interpreter]") {
  Interpreter i(ast("return 123;"));

  REQUIRE(i.run() == 123);
}

TEST_CASE("math can be done", "[interpreter]") {
  Interpreter i(ast("return 100 - 8 * 6 / 2 + 9;"));

  REQUIRE(i.run() == 85);
}
