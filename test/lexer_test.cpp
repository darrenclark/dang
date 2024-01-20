#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_range_equals.hpp>

#include "../src/lexer.h"

using Catch::Matchers::RangeEquals;

TEST_CASE("basic program can be parsed", "[lexer]") {
  Lexer lexer(" return 123; ");

  const std::array<Token, 3> expected{{
      {.type = TokenType::kw_return},
      {.type = TokenType::integer_literal, .value = "123"},
      {.type = TokenType::semicolon},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}
