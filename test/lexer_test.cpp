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

TEST_CASE("keywords can be lexed", "[lexer]") {
  Lexer lexer(" return let if else ");

  const std::array<Token, 4> expected{{
      {.type = TokenType::kw_return},
      {.type = TokenType::kw_let},
      {.type = TokenType::kw_if},
      {.type = TokenType::kw_else},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("literals can be lexed", "[lexer]") {
  Lexer lexer(" 100 3.14");

  const std::array<Token, 2> expected{{
      {.type = TokenType::integer_literal, .value = "100"},
      {.type = TokenType::double_literal, .value = "3.14"},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("math operators can be lexed", "[lexer]") {
  Lexer lexer("5 * (2 + 3) - 4 / 2");

  const std::array<Token, 11> expected{{
      {.type = TokenType::integer_literal, .value = "5"},
      {.type = TokenType::star},
      {.type = TokenType::open_paren},
      {.type = TokenType::integer_literal, .value = "2"},
      {.type = TokenType::plus},
      {.type = TokenType::integer_literal, .value = "3"},
      {.type = TokenType::close_paren},
      {.type = TokenType::minus},
      {.type = TokenType::integer_literal, .value = "4"},
      {.type = TokenType::slash},
      {.type = TokenType::integer_literal, .value = "2"},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("assignment can be lexed", "[lexer]") {
  Lexer lexer("let x = 567;");

  const std::array<Token, 5> expected{{
      {.type = TokenType::kw_let},
      {.type = TokenType::identifier, .value = "x"},
      {.type = TokenType::equals},
      {.type = TokenType::integer_literal, .value = "567"},
      {.type = TokenType::semicolon},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("slash-slash comments are skipped over", "[lexer]") {
  Lexer lexer("let x = // set x to ...\n 567;");

  const std::array<Token, 5> expected{{
      {.type = TokenType::kw_let},
      {.type = TokenType::identifier, .value = "x"},
      {.type = TokenType::equals},
      {.type = TokenType::integer_literal, .value = "567"},
      {.type = TokenType::semicolon},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}

TEST_CASE("slash-star comments are skipped over", "[lexer]") {
  Lexer lexer("let x = /* set x to */ 567;");

  const std::array<Token, 5> expected{{
      {.type = TokenType::kw_let},
      {.type = TokenType::identifier, .value = "x"},
      {.type = TokenType::equals},
      {.type = TokenType::integer_literal, .value = "567"},
      {.type = TokenType::semicolon},
  }};

  CHECK_THAT(lexer.lex(), RangeEquals(expected));
}
