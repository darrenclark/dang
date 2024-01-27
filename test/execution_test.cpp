#include <catch2/catch_test_macros.hpp>

#include "../src/vm.h"

static Value compile_and_run(const std::string &source) {
  VM vm;
  return vm.eval(source);
}

TEST_CASE("basic program can be run", "[execution]") {
  std::string code = "return 123;";

  REQUIRE(compile_and_run(code) == Value::of(123));
}

TEST_CASE("math can be done", "[execution]") {
  std::string code = "return 9 + (16 - 6) / 2 * 9;";

  REQUIRE(compile_and_run(code) == Value::of(54));
}

TEST_CASE("adding an integer and a float produce a float", "[execution]") {
  std::string code = "return 5 * 1.5;";

  REQUIRE(compile_and_run(code) == Value::of(7.5));
}

TEST_CASE("can read and write variables in outer scopes", "[execution]") {
  std::string code = "let x = 5; { x = x * x; } return x;";

  REQUIRE(compile_and_run(code) == Value::of(25));
}

TEST_CASE("can shadow variables in outer scopes", "[execution]") {
  std::string source = "let x = 5; { let x = 2; x = 9; } return x;";

  REQUIRE(compile_and_run(source) == Value::of(5));
}

TEST_CASE("if statement evaluating to true", "[execution]") {
  std::string program = "let x = 5; "
                        "if x { x = x * 5; } "
                        "return x; ";

  REQUIRE(compile_and_run(program) == Value::of(25));
}

TEST_CASE("if statement evaluating to false", "[execution]") {
  std::string program = "let x = 5; "
                        "if x - 5 { x = x * 5; } "
                        "return x; ";

  REQUIRE(compile_and_run(program) == Value::of(5));
}

TEST_CASE("complex if else if changes evaluate correctly", "[execution]") {
  SECTION("if is true") {
    std::string program = "let x = 0; if 1 { x = 1; } else if 1 { x = 2; } "
                          "else { x = 3; } return x;";

    REQUIRE(compile_and_run(program) == Value::of(1));
  }

  SECTION("else if is true") {
    std::string program = "let x = 0; if 0 { x = 1; } else if 1 { x = 2; } "
                          "else { x = 3; } return x;";

    REQUIRE(compile_and_run(program) == Value::of(2));
  }

  SECTION("else is true") {
    std::string program = "let x = 0; if 0 { x = 1; } else if 0 { x = 2; } "
                          "else { x = 3; } return x;";

    REQUIRE(compile_and_run(program) == Value::of(3));
  }
}
