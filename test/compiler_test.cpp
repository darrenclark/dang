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
  Vars vars{CompilerKind::script};

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
    vars.define("a"); // global

    vars.start_scope();
    vars.define("a"); // local @ 0

    REQUIRE(vars.lookup("a") == Vars::Ref{Vars::Local{.index = 0}});
  }

  SECTION("global vs locals when CompilerKind::script") {
    Vars vars(CompilerKind::script);

    SECTION("lookup defaults to global variables if not defined locally") {
      vars.start_scope();

      REQUIRE(vars.lookup("abc") == Vars::Ref{Vars::Global{.name = "abc"}});
    }

    SECTION("variables in root scope are globals") {
      REQUIRE(vars.define("x") == Vars::Ref{Vars::Global{.name = "x"}});

      vars.start_scope();

      REQUIRE(vars.lookup("x") == Vars::Ref{Vars::Global{.name = "x"}});
    }

    SECTION("variables in nested scopes are locals") {
      vars.start_scope();
      REQUIRE(vars.define("x") == Vars::Ref{Vars::Local{.index = 0}});
      REQUIRE(vars.lookup("x") == Vars::Ref{Vars::Local{.index = 0}});

      REQUIRE(vars.define("y") == Vars::Ref{Vars::Local{.index = 1}});
      REQUIRE(vars.lookup("y") == Vars::Ref{Vars::Local{.index = 1}});
    }
  }

  SECTION("global vs locals when CompilerKind::function") {
    Vars vars(CompilerKind::function);

    SECTION("lookup defaults to global variables if not defined locally") {
      vars.start_scope();

      REQUIRE(vars.lookup("abc") == Vars::Ref{Vars::Global{.name = "abc"}});
    }

    SECTION("variables in root scope are locals") {
      REQUIRE(vars.define("x") == Vars::Ref{Vars::Local{.index = 0}});

      vars.start_scope();

      REQUIRE(vars.lookup("x") == Vars::Ref{Vars::Local{.index = 0}});
    }

    SECTION("variables in nested scopes are locals") {
      vars.start_scope();
      REQUIRE(vars.define("x") == Vars::Ref{Vars::Local{.index = 0}});
      REQUIRE(vars.lookup("x") == Vars::Ref{Vars::Local{.index = 0}});

      REQUIRE(vars.define("y") == Vars::Ref{Vars::Local{.index = 1}});
      REQUIRE(vars.lookup("y") == Vars::Ref{Vars::Local{.index = 1}});
    }
  }
}

TEST_CASE("correct bytecode is generated for nested scopes", "[compiler]") {
  std::string source = "let x = 5; { let y = x; x = y * 2; } return x;";

  Chunk compiled = compile(source);

  // clang-format off
  const int expected[] = {
    Op::load_const, 0,
    Op::define_global, 1,
    Op::get_global, 2,
    Op::get_local, 0,
    Op::load_const, 3,
    Op::multiply,
    Op::set_global, 4,
    Op::pop,
    Op::get_global, 5,
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
    Op::define_global, 1,
    Op::get_global, 2,
    Op::jump_if_zero, 7,
    Op::get_global, 3,
    Op::load_const, 4,
    Op::multiply,
    Op::set_global, 5,
    Op::get_global, 6,
    Op::return_
  };
  // clang-format on

  CHECK_THAT(compiled.code, RangeEquals(expected));
}
