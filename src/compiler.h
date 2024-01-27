#pragma once

#include "lexer.h"
#include "parser.h"
#include "value-ptr.hpp"
#include <span>
#include <sstream>
#include <string>
#include <unordered_map>

struct Value {
  std::variant<int, double> value;

  static Value of(int v) { return Value{.value = v}; }

  static Value of(double v) { return Value{.value = v}; }

  bool operator==(const Value &) const = default;

  std::string type() const {
    struct TypeVisitor {
      std::string operator()(int v) const { return "int"; }
      std::string operator()(double v) const { return "double"; }
    };
    return std::visit(TypeVisitor{}, value);
  }

  std::string to_string() const {
    struct ToStringVisitor {
      std::string operator()(int v) const { return std::to_string(v); }
      std::string operator()(double v) const { return std::to_string(v); }
    };
    return std::visit(ToStringVisitor{}, value);
  }

  operator bool() const {
    struct BoolVisitor {
      bool operator()(int v) const { return v != 0; }
      bool operator()(double v) const { return v != 0.0; }
    };
    return std::visit(BoolVisitor{}, value);
  }

  Value &operator+=(const Value &rhs) {
    *this = binaryOp<std::plus<>>(*this, rhs);
    return *this;
  }

  friend Value operator+(Value lhs, const Value &rhs) {
    lhs += rhs;
    return lhs;
  }

  Value &operator-=(const Value &rhs) {
    *this = binaryOp<std::minus<>>(*this, rhs);
    return *this;
  }

  friend Value operator-(Value lhs, const Value &rhs) {
    lhs -= rhs;
    return lhs;
  }

  Value &operator*=(const Value &rhs) {
    *this = binaryOp<std::multiplies<>>(*this, rhs);
    return *this;
  }

  friend Value operator*(Value lhs, const Value &rhs) {
    lhs *= rhs;
    return lhs;
  }

  Value &operator/=(const Value &rhs) {
    *this = binaryOp<std::divides<>>(*this, rhs);
    return *this;
  }

  friend Value operator/(Value lhs, const Value &rhs) {
    lhs /= rhs;
    return lhs;
  }

private:
  template <typename F> Value binaryOp(const Value &lhs, const Value &rhs) {
    return std::visit(
        [&](auto &&a, auto &&b) {
          if constexpr (std::is_same_v<decltype(a), decltype(b)>) {
            return Value{.value = F()(a, b)};
          } else if constexpr (std::is_arithmetic_v<
                                   std::remove_reference_t<decltype(a)>> &&
                               std::is_arithmetic_v<
                                   std::remove_reference_t<decltype(b)>>) {
            return Value{.value = F()((double)a, (double)b)};
          } else {
            std::cerr << "invalid operands: " << lhs.type() << " and "
                      << rhs.type() << std::endl;
            exit(EXIT_FAILURE);
            return Value{};
          }
        },
        lhs.value, rhs.value);
  }
};

inline std::ostream &operator<<(std::ostream &os, Value const &value) {
  os << value.to_string();
  return os;
}

enum Op : int {
  // load_const  X :  pushes constant X from constant table
  load_const,
  // get_local  N :  Gets local variable (N is relative to frame pointer)
  get_local,
  // set_local  N :  Sets local variable to value at top of stack (and pops it)
  set_local,
  // add :  Adds top 2 elements on stack
  add,
  // subtract :  Subtracts top 2 elements on stack (a=pop(); b=pop(); push(b -
  // a))
  subtract,
  // multiply :  Multiplies top 2 elements on stack
  multiply,
  // divide :  Divides top 2 elements on the stack (a=pop(); b=pop(); push(b/a))
  divide,
  // pop : Pops one element off the top of the stack
  pop,
  // jump N : Jumps N instructions
  jump,
  // jump_if_zero N : Jumps N instructions if top of stack is zero (and pops it)
  jump_if_zero,
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
  case set_local:
    return "set_local";
  case add:
    return "add";
  case subtract:
    return "subtract";
  case multiply:
    return "multiply";
  case divide:
    return "divide";
  case pop:
    return "pop";
  case jump:
    return "jump";
  case jump_if_zero:
    return "jump_if_zero";
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
  case set_local:
    return 1;
  case add:
  case subtract:
  case multiply:
  case divide:
  case pop:
    return 0;
  case jump:
  case jump_if_zero:
    return 1;
  case return_:
    return 0;
  case OP_COUNT:
    return 0;
  }

  return 0;
}

class Vars {
public:
  std::optional<int> lookup(const std::string &name) {
    auto it = std::find(vars.rbegin(), vars.rend(), name);
    if (it != vars.rend()) {
      int from_end = it - vars.rbegin();
      return vars.size() - from_end - 1;
    }
    return std::nullopt;
  }

  int define(const std::string &name) {
    auto it = std::find(vars.begin() + *scopes.rbegin(), vars.end(), name);
    if (it != vars.end()) {
      std::cerr << "Variable already defined, cannot redefine: " << name
                << std::endl;
      exit(EXIT_FAILURE);
    }

    int index = vars.size();
    vars.push_back(name);
    return index;
  }

  void start_scope() { scopes.push_back(vars.size()); }

  int end_scope() {
    int count = vars.size() - scopes.at(scopes.size() - 1);
    scopes.pop_back();
    for (int i = 0; i < count; i++) {
      vars.pop_back();
    }
    return count;
  }

private:
  std::vector<std::string> vars;
  std::vector<size_t> scopes{0};
};

struct Chunk {
  std::vector<int> code;
  std::vector<Value> constants;
};

class Compiler {
public:
  Compiler() {}

  Chunk compile(const std::string &source) {
    Lexer lexer(source);
    Parser parser(lexer.lex());
    (*this)(parser.parse());
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

    locals.define(node.identifier.value);
  }

  void operator()(const ASTNodeAssign &node) {
    auto index = locals.lookup(node.identifier.value);
    if (index) {
      (*this)(node.expr);
      chunk.code.push_back(Op::set_local);
      chunk.code.push_back(*index);
    } else {
      std::cerr << "Undefined variable: " << node.identifier.value << std::endl;
      exit(EXIT_FAILURE);
    }
  }

  void operator()(const ASTNodeScope &node) {
    locals.start_scope();

    for (const auto &stmt : node.body) {
      (*this)(stmt);
    }

    int num = locals.end_scope();
    for (int i = 0; i < num; i++) {
      chunk.code.push_back(Op::pop);
    }
  }

  void operator()(const ASTNodeIf &node) {
    std::vector<int> end_jump_offsets;

    (*this)(node.condition);

    chunk.code.push_back(Op::jump_if_zero);

    int i = chunk.code.size();
    chunk.code.push_back(0);

    (*this)(node.body);

    const ASTNodeIf::Rest *rest = &node.rest;
    const ASTNodeIf::Rest rest_end = std::monostate{};

    while (!std::holds_alternative<std::monostate>(*rest)) {
      chunk.code.push_back(Op::jump);
      end_jump_offsets.push_back(chunk.code.size());
      chunk.code.push_back(0);

      // if previous condition was false, jump here
      chunk.code[i] = chunk.code.size() - i - 1;

      std::visit(
          [&](const auto &node) {
            if constexpr (std::is_same<
                              decltype(node),
                              const valuable::value_ptr<ASTNodeElseIf> &>()) {
              (*this)(node->condition);
              chunk.code.push_back(Op::jump_if_zero);

              i = chunk.code.size();
              chunk.code.push_back(0);

              (*this)(node->body);
              rest = &node->rest;

            } else if constexpr (std::is_same<decltype(node),
                                              const valuable::value_ptr<
                                                  ASTNodeElse> &>()) {
              i = -1;

              (*this)(node->body);
              rest = &rest_end;
            }
          },
          *rest);
    }

    for (int offset : end_jump_offsets) {
      chunk.code[offset] = chunk.code.size() - offset - 1;
    }

    if (i >= 0) {
      // if previous condition was false, jump here
      chunk.code[i] = chunk.code.size() - i - 1;
    }
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
    Value value = Value{.value = std::stoi(node.token.value)};

    chunk.constants.push_back(value);
    int index = chunk.constants.size() - 1;

    chunk.code.push_back(Op::load_const);
    chunk.code.push_back(index);
  }

  void operator()(const ASTNodeDoubleLiteral &node) {
    Value value = Value{.value = std::stod(node.token.value)};

    chunk.constants.push_back(value);
    int index = chunk.constants.size() - 1;

    chunk.code.push_back(Op::load_const);
    chunk.code.push_back(index);
  }

  void operator()(const ASTNodeIdentifier &node) {
    auto index = locals.lookup(node.token.value);
    if (index) {
      chunk.code.push_back(Op::get_local);
      chunk.code.push_back(*index);
    } else {
      std::cerr << "Undefined variable: " << node.token.value << std::endl;
      exit(EXIT_FAILURE);
    }
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
  Chunk chunk{};
  Vars locals{};
};
