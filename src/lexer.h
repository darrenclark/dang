#pragma once

#include <cctype>
#include <iostream>
#include <optional>
#include <string>
#include <vector>

enum class TokenType {
  integer_literal,
  kw_return,

  // punctuation
  open_paren,
  close_paren,
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

class Lexer {
public:
  Lexer(const std::string &src) : src(src) {}

  std::vector<Token> lex() {
    std::vector<Token> tokens;

    while (auto ch = peek()) {
      if (std::isspace(*ch)) {
        consume();
      } else if (std::isdigit(*ch)) {
        auto value = consume_while([](char c) { return std::isdigit(c); });
        tokens.push_back({.type = TokenType::integer_literal, .value = value});
      } else if (std::isalpha(*ch)) {
        auto value = consume_while([](char c) { return std::isalnum(c); });
        if (value == "return") {
          tokens.push_back({.type = TokenType::kw_return});
        } else {
          std::cerr << "unexpected word: " << value << std::endl;
          exit(EXIT_FAILURE);
        }
      } else if (*ch == '(') {
        consume();
        tokens.push_back({.type = TokenType::open_paren});
      } else if (*ch == ')') {
        consume();
        tokens.push_back({.type = TokenType::close_paren});
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
