#pragma once

#include "parser.h"
#include "value-ptr.hpp"
#include <string>
#include <unordered_map>

using Value = int;

class Vars {
public:
  void begin_scope() { values.push_back({}); }

  void end_scope() { values.pop_back(); }

  void define(const std::string &name, Value value) {
    auto &top = values.at(values.size() - 1);

    if (top.contains(name)) {
      std::cerr << "error: variable '" << name << "' is already defined"
                << std::endl;
      exit(EXIT_FAILURE);
    }

    top[name] = value;
  }

  void assign(const std::string &name, Value newValue) {
    find_var(name) = newValue;
  }

  Value get(const std::string &name) { return find_var(name); }

private:
  std::vector<std::unordered_map<std::string, Value>> values{};

  Value &find_var(const std::string &variable_name) {
    auto it = std::find_if(values.rbegin(), values.rend(), [&](auto scope) {
      return scope.contains(variable_name);
    });

    if (it == values.rend()) {
      std::cerr << "error: undefined variable '" << variable_name << "'"
                << std::endl;
      exit(EXIT_FAILURE);
    }

    return it->find(variable_name)->second;
  }
};

class Interpreter {
public:
  Interpreter(ASTNodeProgram program) : program(std::move(program)) {}

  int run() { return (*this)(program); }

  int operator()(const ASTNodeProgram &node) {
    int result = 0;

    vars.begin_scope();

    for (const auto &stmt : node.body) {
      result = (*this)(stmt);
    }

    vars.end_scope();

    return result;
  }

  int operator()(const ASTNodeStmt &node) {
    return std::visit(*this, node.child);
  }

  int operator()(const ASTNodeReturn &node) { return (*this)(node.expr); }

  int operator()(const ASTNodeLet &node) {
    auto val = (*this)(node.expr);
    vars.define(node.identifier.value, val);
    return val;
  }

  int operator()(const ASTNodeAssign &node) {
    auto val = (*this)(node.expr);
    vars.assign(node.identifier.value, val);
    return val;
  }

  int operator()(const ASTNodeScope &node) {
    int result = 0;

    vars.begin_scope();

    for (const auto &stmt : node.body) {
      result = (*this)(stmt);
    }

    vars.end_scope();

    return result;
  }

  int operator()(const ASTNodeIf &node) {
    auto condition = (*this)(node.condition);

    if (condition) {
      return (*this)(node.body);
    } else {
      return 0;
    }
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
    return vars.get(node.token.value);
  }

  int operator()(const ASTNodeParenExpr &node) { return (*this)(node.child); }

  template <typename T> int operator()(const valuable::value_ptr<T> &ptr) {
    return (*this)(*ptr);
  }

private:
  ASTNodeProgram program;
  Vars vars;
};
