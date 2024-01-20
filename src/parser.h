#pragma once

#include "lexer.h"
#include "value-ptr.hpp"
#include <memory>
#include <variant>

enum class BinOp { add, subtract, multiply, divide };

inline int bin_op_prec(BinOp op) {
  switch (op) {
  case BinOp::add:
  case BinOp::subtract:
    return 0;
  case BinOp::multiply:
  case BinOp::divide:
    return 1;
  }
}

inline std::optional<BinOp> bin_op_for_token(TokenType type) {
  switch (type) {
  case TokenType::plus:
    return BinOp::add;
  case TokenType::minus:
    return BinOp::subtract;
  case TokenType::star:
    return BinOp::multiply;
  case TokenType::slash:
    return BinOp::divide;
  default:
    return std::nullopt;
  }
}

inline std::string to_string(BinOp op) {
  switch (op) {
  case BinOp::add:
    return "add";
  case BinOp::subtract:
    return "subtract";
  case BinOp::multiply:
    return "multiply";
  case BinOp::divide:
    return "divide";
  }
}

struct ASTNodeIntegerLiteral {
  Token token;

  bool operator==(const ASTNodeIntegerLiteral &) const = default;
};

struct ASTNodeIdentifier {
  Token token;

  bool operator==(const ASTNodeIdentifier &) const = default;
};

struct ASTNodeParenExpr;

struct ASTNodeTerm {
  std::variant<ASTNodeIntegerLiteral, ASTNodeIdentifier,
               valuable::value_ptr<ASTNodeParenExpr>>
      child;

  bool operator==(const ASTNodeTerm &) const = default;
};

struct ASTNodeExpr;

struct ASTNodeParenExpr {
  valuable::value_ptr<ASTNodeExpr> child;

  bool operator==(const ASTNodeParenExpr &) const = default;
};

struct ASTNodeBinExpr {
  valuable::value_ptr<ASTNodeExpr> lhs;
  valuable::value_ptr<ASTNodeExpr> rhs;
  BinOp op;

  bool operator==(const ASTNodeBinExpr &) const = default;
};

struct ASTNodeExpr {
  std::variant<ASTNodeTerm, ASTNodeBinExpr> child;

  bool operator==(const ASTNodeExpr &) const = default;
};

struct ASTNodeReturn {
  ASTNodeExpr expr;

  bool operator==(const ASTNodeReturn &) const = default;
};

struct ASTNodeLet {
  Token identifier;
  ASTNodeExpr expr;

  bool operator==(const ASTNodeLet &) const = default;
};

struct ASTNodeAssign {
  Token identifier;
  ASTNodeExpr expr;

  bool operator==(const ASTNodeAssign &) const = default;
};

struct ASTNodeScope;
struct ASTNodeIf;

struct ASTNodeStmt {
  std::variant<ASTNodeReturn, ASTNodeLet, ASTNodeAssign,
               valuable::value_ptr<ASTNodeScope>,
               valuable::value_ptr<ASTNodeIf>>
      child;

  bool operator==(const ASTNodeStmt &) const = default;
};

struct ASTNodeScope {
  std::vector<ASTNodeStmt> body;

  bool operator==(const ASTNodeScope &) const = default;
};

struct ASTNodeIf {
  ASTNodeExpr condition;
  ASTNodeScope body;

  bool operator==(const ASTNodeIf &) const = default;
};

struct ASTNodeProgram {
  std::vector<ASTNodeStmt> body;

  bool operator==(const ASTNodeProgram &) const = default;
};

class Parser {
public:
  Parser(const std::vector<Token> &tokens) : tokens(tokens) {}

  ASTNodeProgram parse() {
    std::vector<ASTNodeStmt> body;

    while (peek()) {
      if (auto stmt = parse_stmt()) {
        body.push_back(*stmt);
      } else {
        std::cerr << "expected statement" << std::endl;
        exit(EXIT_FAILURE);
      }
    }

    return {.body = body};
  }

  std::optional<ASTNodeStmt> parse_stmt() {
    auto token = peek();
    if (!token)
      return std::nullopt;

    if (token->type == TokenType::kw_return) {
      consume();

      auto expr = parse_expr();
      if (!expr) {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }
      must_consume(TokenType::semicolon, "expected `;`");

      return {{.child = (ASTNodeReturn){.expr = *expr}}};
    } else if (token->type == TokenType::kw_let) {
      consume();

      auto identifier =
          must_consume(TokenType::identifier, "expected identifier");
      must_consume(TokenType::equals, "expected `=`");
      auto expr = parse_expr();
      if (!expr) {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }
      must_consume(TokenType::semicolon, "expected `;`");

      return {{.child = (ASTNodeLet){.identifier = identifier, .expr = *expr}}};
    } else if (token->type == TokenType::identifier && peek(1) &&
               peek(1)->type == TokenType::equals) {
      auto identifier = consume();
      consume();

      auto expr = parse_expr();
      if (!expr) {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }
      must_consume(TokenType::semicolon, "expected `;`");

      return {
          {.child = (ASTNodeAssign){.identifier = identifier, .expr = *expr}}};
    } else if (token->type == TokenType::open_curly) {
      auto scope = parse_scope();
      if (!scope) {
        std::cerr << "expected scope" << std::endl;
        exit(EXIT_FAILURE);
      }
      return {{.child = *scope}};
    } else if (token->type == TokenType::kw_if) {
      consume();

      auto condition = parse_expr();
      if (!condition) {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }

      auto body = parse_scope();
      if (!body) {
        std::cerr << "expected scope for if statement body" << std::endl;
        exit(EXIT_FAILURE);
      }

      return {{.child = (ASTNodeIf){.condition = *condition, .body = *body}}};
    }

    return std::nullopt;
  }

  std::optional<ASTNodeScope> parse_scope() {
    if (!peek() || peek()->type != TokenType::open_curly) {
      return std::nullopt;
    }
    consume();

    std::vector<ASTNodeStmt> body;

    while (peek() && peek()->type != TokenType::close_curly) {
      if (auto stmt = parse_stmt()) {
        body.push_back(*stmt);
      } else {
        std::cerr << "expected statement" << std::endl;
        exit(EXIT_FAILURE);
      }
    }

    must_consume(TokenType::close_curly, "expected `}`");

    return {{.body = body}};
  }

  std::optional<ASTNodeExpr> parse_expr(int min_prec = 0) {
    // Based on
    // https://github.com/orosmatthew/hydrogen-cpp/blob/1e2d30d2c0f75313890f8faba78cedecf144b14e/src/parser.hpp#L144

    auto term_lhs = parse_term();
    if (!term_lhs)
      return std::nullopt;

    ASTNodeExpr expr_lhs{*term_lhs};

    while (true) {
      auto current_token = peek();
      if (!current_token)
        break;

      auto bin_op = bin_op_for_token(current_token->type);
      if (!bin_op)
        break;

      auto prec = bin_op_prec(*bin_op);
      if (prec < min_prec)
        break;

      consume();

      int next_min_prec = prec + 1;
      auto expr_rhs = parse_expr(next_min_prec);

      if (!expr_rhs) {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }

      ASTNodeBinExpr bin_expr = {
          .lhs = expr_lhs, .rhs = *expr_rhs, .op = *bin_op};

      expr_lhs = {bin_expr};
    }

    return expr_lhs;
  }

  std::optional<ASTNodeTerm> parse_term() {
    auto token = peek();
    if (!token)
      return std::nullopt;

    if (token->type == TokenType::integer_literal) {
      return {{.child = (ASTNodeIntegerLiteral){.token = consume()}}};
    } else if (token->type == TokenType::identifier) {
      return {{.child = (ASTNodeIdentifier){.token = consume()}}};
    } else if (token->type == TokenType::open_paren) {
      consume();
      if (auto expr = parse_expr()) {
        must_consume(TokenType::close_paren, "expected `)`");
        return {{.child = (ASTNodeParenExpr){.child = *expr}}};
      } else {
        std::cerr << "expected expression" << std::endl;
        exit(EXIT_FAILURE);
      }
    }
    // TODO: Variables/etc.
    return std::nullopt;
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
