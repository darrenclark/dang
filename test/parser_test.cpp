#include <catch2/catch_test_macros.hpp>

#include "../src/lexer.h"
#include "../src/parser.h"

static std::vector<Token> tokens(std::string input) {
  Lexer l(input);
  return l.lex();
}

TEST_CASE("basic program can be parsed", "[parser]") {

  Parser p(tokens("return 123;"));

  ASTNodeProgram expected = {
      .body = {
          .expr = {(ASTNodeTerm){
              .integer_literal = {.token = {.type = TokenType::integer_literal,
                                            .value = "123"}}}}}};

  REQUIRE(p.parse() == expected);
}
