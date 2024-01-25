#pragma once

#include "parser.h"
#include "value-ptr.hpp"
#include <span>
#include <string>
#include <unordered_map>

using Value = int;

enum Op : int {
  // load_const  X :  pushes constant X from constant table
  load_const,
  // get_local  N :  Gets local variable (N is relative to frame pointer)
  get_local,
  // add :  Adds top 2 elements on stack
  add,
  // subtract :  Subtracts top 2 elements on stack (a=pop(); b=pop(); push(b -
  // a))
  subtract,
  // multiply :  Multiplies top 2 elements on stack
  multiply,
  // divide :  Divides top 2 elements on the stack (a=pop(); b=pop(); push(b/a))
  divide,
  // return :  Returns top value on stack
  return_,

  // constant for
  OP_COUNT
};

inline std::string to_string(Op op) {
  switch (op) {
  case load_const:
    return "load_const";
  case get_local:
    return "get_local";
  case add:
    return "add";
  case subtract:
    return "subtract";
  case multiply:
    return "multiply";
  case divide:
    return "divide";
  case return_:
    return "return_";
  case OP_COUNT:
    return "<invalid>";
  }
  return "<invalid>";
}

inline int op_n_args(Op op) {
  switch (op) {
  case load_const:
    return 1;
  case get_local:
    return 1;
  case add:
  case subtract:
  case multiply:
  case divide:
    return 0;
  case return_:
    return 0;
  case OP_COUNT:
    return 0;
  }

  return 0;
}

struct Chunk {
  std::vector<int> code;
  std::vector<Value> constants;
};

class Compiler {
public:
  Compiler(ASTNodeProgram program) : program(std::move(program)) {}

  Chunk compile() {
    (*this)(program);
    return chunk;
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

    chunk.code.push_back(Op::return_);
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
      chunk.code.push_back(Op::add);
      return;
    case BinOp::subtract:
      chunk.code.push_back(Op::subtract);
      return;
    case BinOp::multiply:
      chunk.code.push_back(Op::multiply);
      return;
    case BinOp::divide:
      chunk.code.push_back(Op::divide);
      return;
    }
  }

  void operator()(const ASTNodeIntegerLiteral &node) {
    int value = std::stoi(node.token.value);

    chunk.constants.push_back(value);
    int index = chunk.constants.size() - 1;

    chunk.code.push_back(Op::load_const);
    chunk.code.push_back(index);
  }

  void operator()(const ASTNodeIdentifier &node) {
    auto it = std::find(vars.begin(), vars.end(), node.token.value);
    assert(it != vars.end());

    chunk.code.push_back(Op::get_local);
    chunk.code.push_back(it - vars.begin());
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
  Chunk chunk{};
  std::vector<std::string> vars{};
};
