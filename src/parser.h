#pragma once

#include "lexer.h"

struct ASTNodeReturn {
  Token integer_literal;

  bool operator==(const ASTNodeReturn &) const = default;
};

struct ASTNodeProgram {
  ASTNodeReturn body;

  bool operator==(const ASTNodeProgram &) const = default;
};

class Parser {
public:
  Parser(const std::vector<Token> &tokens) : tokens(tokens) {}

  ASTNodeProgram parse() {
    must_consume(TokenType::kw_return, "expected `return`");
    auto integer_literal =
        must_consume(TokenType::integer_literal, "expected integer literal");
    must_consume(TokenType::semicolon, "expected integer literal");

    if (peek()) {
      std::cerr << "expected EOF" << std::endl;
      exit(EXIT_FAILURE);
    }

    return {.body = {.integer_literal = integer_literal}};
  }

private:
  [[nodiscard]] std::optional<Token> peek(int offset = 0) const {
    if (current_position + offset >= tokens.size()) {
      return std::nullopt;
    } else {
      return tokens.at(current_position + offset);
    }
  }

  Token consume() { return tokens.at(current_position++); }

  Token must_consume(TokenType type, const char *error_message) {
    auto t = peek();
    if (t && t->type == type) {
      consume();
      return *t;
    } else {
      std::cerr << error_message << std::endl;
      exit(EXIT_FAILURE);
    }
  }

  std::vector<Token> tokens;
  size_t current_position = 0;
};
