#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_range_equals.hpp>

#include "../src/compiler.h"
#include "../src/disassembler.h"

using Catch::Matchers::RangeEquals;

static Chunk compile(const std::string &source) {
  Compiler c{};
  return c.compile(source);
}

TEST_CASE("Vars", "[compiler]") {
  Vars vars{};

  SECTION("end_scope returns number of variables to pop") {
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

  SECTION("shadowing variables works") {
    vars.define("a");

    vars.start_scope();
    vars.define("a");

    REQUIRE(vars.lookup("a") == 1);
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

TEST_CASE("correct bytecode is generated for if statement", "[compiler]") {
  std::string source = "let x = 5; "
                       "if x { x = x * 5; } "
                       "return x; ";

  Chunk compiled = compile(source);

  // clang-format off
  const int expected[] = {
    Op::load_const, 0,
    Op::get_local, 0,
    Op::jump_if_zero, 7,
    Op::get_local, 0,
    Op::load_const, 1,
    Op::multiply,
    Op::set_local, 0,
    Op::get_local, 0,
    Op::return_
  };
  // clang-format on

  Disassembler d;
  std::cerr << d.disassemble(compiled) << std::endl;

  CHECK_THAT(compiled.code, RangeEquals(expected));
}
