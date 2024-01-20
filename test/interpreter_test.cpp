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
  Interpreter i(ast("return (100 - 8) * 6 / 2 + 9;"));

  REQUIRE(i.run() == 285);
}

TEST_CASE("can read and write variables in outer scopes", "[interpreter]") {
  Interpreter i(ast("let x = 5; { x = x + x; } return x;"));

  REQUIRE(i.run() == 10);
}

TEST_CASE("can shadow variables in outer scopes", "[interpreter]") {
  Interpreter i(ast("let x = 5; { let x = 2; x = 9; } return x;"));

  REQUIRE(i.run() == 5);
}

TEST_CASE("if statement evaluating to true", "[interpreter]") {
  std::string program = "let x = 5; "
                        "if x { x = x * 5; } "
                        "return x; ";

  Interpreter i(ast(program));

  REQUIRE(i.run() == 25);
}

TEST_CASE("if statement evaluating to false", "[interpreter]") {
  std::string program = "let x = 5; "
                        "if x - 5 { x = x * 5; } "
                        "return x; ";

  Interpreter i(ast(program));

  REQUIRE(i.run() == 5);
}
