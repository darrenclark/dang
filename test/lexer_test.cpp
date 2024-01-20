#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_range_equals.hpp>

#include "../src/lexer.h"

using Catch::Matchers::RangeEquals;

TEST_CASE("basic program can be lexed", "[lexer]") {
  Lexer lexer(" return 123; ");

  const std::array<Token, 3> expected{{
      {.type = TokenType::kw_return},
      {.type = TokenType::integer_literal, .value = "123"},
      {.type = TokenType::semicolon},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("math operators can be lexed", "[lexer]") {
  Lexer lexer("5 * 2 + 3 - 4 / 2");

  const std::array<Token, 9> expected{{
      {.type = TokenType::integer_literal, .value = "5"},
      {.type = TokenType::star},
      {.type = TokenType::integer_literal, .value = "2"},
      {.type = TokenType::plus},
      {.type = TokenType::integer_literal, .value = "3"},
      {.type = TokenType::minus},
      {.type = TokenType::integer_literal, .value = "4"},
      {.type = TokenType::slash},
      {.type = TokenType::integer_literal, .value = "2"},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}
