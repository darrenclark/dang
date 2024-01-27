#include <catch2/catch_test_macros.hpp>

#include "../src/compiler.h"
#include "../src/vm.h"

static int compile_and_run(const std::string &source) {
  Lexer l(source);
  Parser p(l.lex());
  Compiler c(p.parse());
  auto code = c.compile();
  VM vm(code);
  return vm.run();
}

TEST_CASE("basic program can be run", "[execution]") {
  std::string code = "return 123;";

  REQUIRE(compile_and_run(code) == 123);
}

TEST_CASE("math can be done", "[execution]") {
  std::string code = "return 9 + (16 - 6) / 2 * 9;";

  REQUIRE(compile_and_run(code) == 54);
}

TEST_CASE("can read and write variables in outer scopes", "[execution]") {
  std::string code = "let x = 5; { x = x * x; } return x;";

  REQUIRE(compile_and_run(code) == 25);
}

/*
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
*/
