#pragma once

#include "parser.h"
#include "value-ptr.hpp"
#include <string>
#include <unordered_map>

class Interpreter {
public:
  Interpreter(ASTNodeProgram program) : program(std::move(program)) {}

  int run() { return (*this)(program); }

  int operator()(const ASTNodeProgram &node) {
    int result = 0;
    for (const auto &stmt : node.body) {
      result = (*this)(stmt);
    }
    return result;
  }

  int operator()(const ASTNodeStmt &node) {
    return std::visit(*this, node.child);
  }

  int operator()(const ASTNodeReturn &node) { return (*this)(node.expr); }

  int operator()(const ASTNodeLet &node) {
    auto val = (*this)(node.expr);

    if (vars.contains(node.identifier.value)) {
      std::cerr << "error: variable '" << node.identifier.value
                << "' is already defined" << std::endl;
      exit(EXIT_FAILURE);
    }

    vars[node.identifier.value] = val;

    return val;
  }

  int operator()(const ASTNodeAssign &node) {
    auto val = (*this)(node.expr);

    auto it = vars.find(node.identifier.value);
    if (it == vars.end()) {
      std::cerr << "error: undefined variable '" << node.identifier.value << "'"
                << std::endl;
      exit(EXIT_FAILURE);
    }

    it->second = val;

    return val;
  }

  int operator()(const ASTNodeExpr &node) {
    return std::visit(*this, node.child);
  }

  int operator()(const ASTNodeTerm &node) {
    return std::visit(*this, node.child);
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

  int operator()(const ASTNodeIdentifier &node) {
    auto it = vars.find(node.token.value);

    if (it == vars.end()) {
      std::cerr << "error: undefined variable '" << node.token.value << "'"
                << std::endl;
      exit(EXIT_FAILURE);
    }

    return it->second;
  }

  int operator()(const ASTNodeParenExpr &node) { return (*this)(node.child); }

  template <typename T> int operator()(const valuable::value_ptr<T> &ptr) {
    return (*this)(*ptr);
  }

private:
  ASTNodeProgram program;
  std::unordered_map<std::string, int> vars;
};
