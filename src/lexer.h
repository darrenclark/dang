#pragma once

#include <cctype>
#include <iostream>
#include <optional>
#include <string>
#include <vector>

enum class TokenType {
  integer_literal,
  identifier,
  kw_return,
  kw_let,

  // punctuation
  equals,
  open_paren,
  close_paren,
  open_curly,
  close_curly,
  minus,
  plus,
  semicolon,
  slash,
  star,
};

struct Token {
  TokenType type{};
  std::string value{};

  bool operator==(const Token &) const = default;
};

inline std::string to_string(TokenType type) {
  switch (type) {

  case TokenType::integer_literal:
    return "integer_literal";
  case TokenType::identifier:
    return "identifier";
  case TokenType::kw_return:
    return "kw_return";
  case TokenType::kw_let:
    return "kw_let";
  case TokenType::equals:
    return "equals";
  case TokenType::open_paren:
    return "open_paren";
  case TokenType::close_paren:
    return "close_paren";
  case TokenType::open_curly:
    return "open_curly";
  case TokenType::close_curly:
    return "close_curly";
  case TokenType::minus:
    return "minus";
  case TokenType::plus:
    return "plus";
  case TokenType::semicolon:
    return "semicolon";
  case TokenType::slash:
    return "slash";
  case TokenType::star:
    return "star";
  }
}

class Lexer {
public:
  Lexer(const std::string &src) : src(src) {}

  std::vector<Token> lex() {
    std::vector<Token> tokens;

    while (auto ch = peek()) {
      if (std::isspace(*ch)) {
        consume();
      } else if (*ch == '/' && peek(1) == '/') {
        consume();
        consume();
        while (peek() != '\n') {
          consume();
        }
      } else if (*ch == '/' && peek(1) == '*') {
        consume();
        consume();

        while (peek() && peek() != '*' && peek(1) != '/') {
          consume();
        }

        if (peek()) {
          consume();
          consume();
        }
      } else if (std::isdigit(*ch)) {
        auto value = consume_while([](char c) { return std::isdigit(c); });
        tokens.push_back({.type = TokenType::integer_literal, .value = value});
      } else if (std::isalpha(*ch)) {
        auto value = consume_while([](char c) { return std::isalnum(c); });
        if (value == "return") {
          tokens.push_back({.type = TokenType::kw_return});
        } else if (value == "let") {
          tokens.push_back({.type = TokenType::kw_let});
        } else {
          tokens.push_back({.type = TokenType::identifier, .value = value});
        }
      } else if (*ch == '=') {
        consume();
        tokens.push_back({.type = TokenType::equals});
      } else if (*ch == '(') {
        consume();
        tokens.push_back({.type = TokenType::open_paren});
      } else if (*ch == ')') {
        consume();
        tokens.push_back({.type = TokenType::close_paren});
      } else if (*ch == '{') {
        consume();
        tokens.push_back({.type = TokenType::open_curly});
      } else if (*ch == '}') {
        consume();
        tokens.push_back({.type = TokenType::close_curly});
      } else if (*ch == '-') {
        consume();
        tokens.push_back({.type = TokenType::minus});
      } else if (*ch == '+') {
        consume();
        tokens.push_back({.type = TokenType::plus});
      } else if (*ch == ';') {
        consume();
        tokens.push_back({.type = TokenType::semicolon});
      } else if (*ch == '/') {
        consume();
        tokens.push_back({.type = TokenType::slash});
      } else if (*ch == '*') {
        consume();
        tokens.push_back({.type = TokenType::star});
      } else {
        std::cerr << "unexpected character: " << *ch << std::endl;
        exit(EXIT_FAILURE);
      }
    }

    return tokens;
  }

private:
  [[nodiscard]] std::optional<char> peek(int offset = 0) const {
    if (current_position + offset >= src.length()) {
      return std::nullopt;
    } else {
      return src.at(current_position + offset);
    }
  }

  char consume() { return src.at(current_position++); }

  template <typename F> std::string consume_while(F f) {
    std::string result;
    while (auto ch = peek()) {
      if (f(*ch)) {
        result += consume();
      } else {
        break;
      }
    }
    return result;
  }

  std::string src;
  size_t current_position = 0;
};
