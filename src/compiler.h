#pragma once

#include "parser.h"
#include "value-ptr.hpp"
#include "vm.h"
#include <span>
#include <string>
#include <unordered_map>

class Compiler {
public:
  Compiler(ASTNodeProgram program) : program(std::move(program)) {}

  std::span<const int> compile() {
    (*this)(program);
    return code;
  }

  void operator()(const ASTNodeProgram &node) {
    // int result = 0;

    // vars.begin_scope();

    for (const auto &stmt : node.body) {
      (*this)(stmt);
    }

    // vars.end_scope();
  }

  void operator()(const ASTNodeStmt &node) {
    return std::visit(*this, node.child);
  }

  void operator()(const ASTNodeReturn &node) {
    (*this)(node.expr);

    code.push_back(Op::return_);
  }

  void operator()(const ASTNodeLet &node) {
    (*this)(node.expr);

    // TODO: Verify variable not already defined

    vars.push_back(node.identifier.value);

    // auto val = ;
    // vars.define(node.identifier.value, val);
    // return val;
  }

  void operator()(const ASTNodeAssign &node) {
    assert(false);
    // auto val = (*this)(node.expr);
    // vars.assign(node.identifier.value, val);
    // return val;
  }

  void operator()(const ASTNodeScope &node) {
    assert(false);
    /*int result = 0;

    vars.begin_scope();

    for (const auto &stmt : node.body) {
      result = (*this)(stmt);
    }

    vars.end_scope();

    return result;*/
  }

  void operator()(const ASTNodeIf &node) {
    assert(false);
    /*auto condition = (*this)(node.condition);

    if (condition) {
      return (*this)(node.body);
    } else {
      return std::visit(*this, node.rest);
    }*/
  }

  void operator()(const ASTNodeElseIf &node) {
    assert(false);
    /*auto condition = (*this)(node.condition);

    if (condition) {
      return (*this)(node.body);
    } else {
      return std::visit(*this, node.rest);
    }*/
  }

  void operator()(const ASTNodeElse &node) {
    assert(false);
    // return (*this)(node.body);
  }

  void operator()(const std::monostate &node) {}

  void operator()(const ASTNodeFunctionDef &node) {
    assert(false);
    // TODO: Implement
  }

  void operator()(const ASTNodeExpr &node) {
    return std::visit(*this, node.child);
  }

  void operator()(const ASTNodeTerm &node) {
    return std::visit(*this, node.child);
  }

  void operator()(const ASTNodeBinExpr &node) {
    (*this)(*node.lhs);
    (*this)(*node.rhs);

    switch (node.op) {
    case BinOp::add:
      assert(false);
      return;
    case BinOp::subtract:
      assert(false);
      return;
    case BinOp::multiply:
      code.push_back(Op::multiply);
      return;
    case BinOp::divide:
      assert(false);
      return;
    }
  }

  void operator()(const ASTNodeIntegerLiteral &node) {
    int value = std::stoi(node.token.value);

    code.push_back(Op::const_int);
    code.push_back(value);
  }

  void operator()(const ASTNodeIdentifier &node) {
    auto it = std::find(vars.begin(), vars.end(), node.token.value);
    assert(it != vars.end());

    code.push_back(Op::get_local);
    code.push_back(it - vars.begin());
  }

  void operator()(const ASTNodeParenExpr &node) { return (*this)(node.child); }

  void operator()(const ASTNodeFunctionCall &node) {
    assert(false);
    // TODO: Implement
  }

  template <typename T> void operator()(const valuable::value_ptr<T> &ptr) {
    return (*this)(*ptr);
  }

private:
  ASTNodeProgram program;
  std::vector<int> code{};
  std::vector<std::string> vars{};
};
