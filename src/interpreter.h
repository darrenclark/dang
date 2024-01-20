#pragma once

#include "parser.h"
#include <string>

class Interpreter {
public:
  Interpreter(ASTNodeProgram program) : program(std::move(program)) {}

  int run() { return (*this)(program); }

  int operator()(const ASTNodeProgram &node) { return (*this)(node.body); }

  int operator()(const ASTNodeReturn &node) { return (*this)(node.expr); }

  int operator()(const ASTNodeExpr &node) {
    return std::visit(*this, node.child);
  }

  int operator()(const ASTNodeTerm &node) {
    return (*this)(node.integer_literal);
  }

  int operator()(const ASTNodeBinExpr &node) {
    auto lhs = (*this)(*node.lhs);
    auto rhs = (*this)(*node.rhs);

    switch (node.op) {
    case BinOp::add:
      return lhs + rhs;
    case BinOp::subtract:
      return lhs - rhs;
    case BinOp::multiply:
      return lhs * rhs;
    case BinOp::divide:
      return lhs / rhs;
    }
  }

  int operator()(const ASTNodeIntegerLiteral &node) {
    return std::stoi(node.token.value);
  }

private:
  ASTNodeProgram program;
};
