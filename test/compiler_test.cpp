#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_range_equals.hpp>

#include "../src/compiler.h"

using Catch::Matchers::RangeEquals;

static Chunk compile(const std::string &source) {
  Lexer l(source);
  Parser p(l.lex());
  Compiler c(p.parse());
  return c.compile();
}

TEST_CASE("Vars end_scope returns number of variables to pop", "[compiler]") {
  Vars vars{};

  SECTION("when nested") {
    vars.define("a");

    vars.start_scope();
    vars.define("b1");

    vars.start_scope();
    vars.define("c");

    vars.start_scope();
    REQUIRE(vars.end_scope() == 0);

    REQUIRE(vars.end_scope() == 1);

    vars.define("b2");
    vars.define("b3");

    REQUIRE(vars.end_scope() == 3);
  }
}

TEST_CASE("correct bytecode is generated for nested scopes", "[compiler]") {
  std::string source = "let x = 5; { let y = x; x = y * 2; } return x;";

  Chunk compiled = compile(source);

  // clang-format off
  const int expected[] = {
    Op::load_const, 0,
    Op::get_local, 0,
    Op::get_local, 1,
    Op::load_const, 1,
    Op::multiply,
    Op::set_local, 0,
    Op::pop,
    Op::get_local, 0,
    Op::return_
  };
  // clang-format on

  CHECK_THAT(compiled.code, RangeEquals(expected));
}
