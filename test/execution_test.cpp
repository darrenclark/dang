#include <catch2/catch_test_macros.hpp>

#include "../src/compiler.h"
#include "../src/vm.h"

static int compile_and_run(const std::string &source) {
  Lexer l(source);
  Parser p(l.lex());
  Compiler c(p.parse());
  auto code = c.compile();
  VM vm(code.data());
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
