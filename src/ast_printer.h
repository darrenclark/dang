#pragma once

#include "parser.h"
#include "value-ptr.hpp"
#include <sstream>
#include <string>

template <typename Root> class ASTPrinter {
public:
  ASTPrinter(Root root) : root(std::move(root)) {}

  std::string print() {
    (*this)(root);
    return output.str();
  }

  void operator()(const ASTNodeProgram &node) {
    begin_struct("ASTNodeProgram");
    begin_vector_field("body");

    for (const auto &stmt : node.body) {
      (*this)(stmt);
      put_indent();
      output << ",\n";
    }

    end_vector_field();
    end_struct();
  }

  void operator()(const ASTNodeStmt &node) {
    begin_struct("ASTNodeStmt");
    begin_variant_field("child");

    std::visit(*this, node.child);

    end_variant_field();
    end_struct();
  }

  void operator()(const ASTNodeReturn &node) {
    begin_struct("ASTNodeReturn");
    begin_field("expr");
    (*this)(node.expr);
    end_field();
    end_struct();
  }

  void operator()(const ASTNodeLet &node) {
    begin_struct("ASTNodeLet");
    field("identifier", node.identifier);
    begin_field("expr");
    (*this)(node.expr);
    end_field();
    end_struct();
  }

  void operator()(const ASTNodeAssign &node) {
    begin_struct("ASTNodeAssign");
    field("identifier", node.identifier);
    begin_field("expr");
    (*this)(node.expr);
    end_field();
    end_struct();
  }

  void operator()(const ASTNodeScope &node) {
    begin_struct("ASTNodeScope");
    begin_vector_field("body");

    for (const auto &stmt : node.body) {
      (*this)(stmt);
      put_indent();
      output << ",\n";
    }

    end_vector_field();
    end_struct();
  }

  void operator()(const ASTNodeIf &node) {
    begin_struct("ASTNodeIf");

    begin_field("condition");
    (*this)(node.condition);
    end_field();

    begin_field("body");
    (*this)(node.body);
    end_field();

    if (std::holds_alternative<std::monostate>(node.rest)) {
      output << ".rest = std::monostate{},\n";
    } else {
      begin_variant_field("rest");
      std::visit(*this, node.rest);
      end_variant_field();
    }

    end_struct();
  }

  void operator()(const ASTNodeElseIf &node) {
    begin_struct("ASTNodeElseIf");

    begin_field("condition");
    (*this)(node.condition);
    end_field();

    begin_field("body");
    (*this)(node.body);
    end_field();

    if (std::holds_alternative<std::monostate>(node.rest)) {
      output << ".rest = std::monostate{},\n";
    } else {
      begin_variant_field("rest");
      std::visit(*this, node.rest);
      end_variant_field();
    }

    end_struct();
  }

  void operator()(const ASTNodeFunctionDef &node) {
    begin_struct("ASTNodeFunctionDef");

    field("name", node.name);
    field("arg_names", node.arg_names);

    begin_field("body");
    (*this)(node.body);
    end_field();

    end_struct();
  }

  void operator()(const ASTNodeElse &node) {
    begin_struct("ASTNodeElse");

    begin_field("body");
    (*this)(node.body);
    end_field();

    end_struct();
  }

  void operator()(const ASTNodeExpr &node) {
    begin_struct("ASTNodeExpr");
    begin_variant_field("child");
    std::visit(*this, node.child);
    end_variant_field();
    end_struct();
  }

  void operator()(const ASTNodeTerm &node) {
    begin_struct("ASTNodeTerm");
    begin_variant_field("child");
    std::visit(*this, node.child);
    end_variant_field();
    end_struct();
  }

  void operator()(const ASTNodeBinExpr &node) {
    begin_struct("ASTNodeBinExpr");

    begin_field("lhs");
    (*this)(node.lhs);
    end_field();

    begin_field("rhs");
    (*this)(node.rhs);
    end_field();

    put_indent();
    output << ".op = BinOp::" << to_string(node.op) << ",\n";

    end_struct();
  }

  void operator()(const ASTNodeIntegerLiteral &node) {
    begin_struct("ASTNodeIntegerLiteral");
    field("token", node.token);
    end_struct();
  }

  void operator()(const ASTNodeDoubleLiteral &node) {
    begin_struct("ASTNodeDoubleLiteral");
    field("token", node.token);
    end_struct();
  }

  void operator()(const ASTNodeIdentifier &node) {
    begin_struct("ASTNodeIdentifier");
    field("token", node.token);
    end_struct();
  }

  void operator()(const ASTNodeParenExpr &node) {
    begin_struct("ASTNodeParenExpr");
    begin_field("child");
    (*this)(node.child);
    end_field();
    end_struct();
  }

  void operator()(const ASTNodeFunctionCall &node) {
    begin_struct("ASTNodeFunctionCall");
    field("name", node.name);

    begin_vector_field("body");

    for (const auto &argument : node.arguments) {
      (*this)(argument);
      put_indent();
      output << ",\n";
    }

    end_vector_field();

    end_struct();
  }

  void operator()(const std::monostate &node) {
    // printing of monostate handled nicer elsewhere
  }

  template <typename T> void operator()(const valuable::value_ptr<T> &ptr) {
    return (*this)(*ptr);
  }

private:
  void begin_struct(const std::string &name) {
    put_indent();
    output << "(" << name << "){\n";

    indent += 1;
  }

  void end_struct() {
    indent -= 1;

    put_indent();
    output << "}\n";
  }

  void begin_vector_field(const std::string &name) {
    put_indent();
    output << "." << name << " = {\n";

    indent += 1;
  }

  void end_vector_field() {
    indent -= 1;

    put_indent();
    output << "}\n";
  }

  void begin_variant_field(const std::string &name) {
    put_indent();
    output << "." << name << " = {\n";

    indent += 1;
  }

  void end_variant_field() {
    indent -= 1;

    put_indent();
    output << "}\n";
  }

  void begin_field(const std::string &name) {
    put_indent();
    output << "." << name << " = \n";

    indent += 1;
  }

  void end_field() {
    put_indent();
    output << ",\n";

    indent -= 1;
  }

  void field(const std::string &name, const Token &token) {
    put_indent();
    output << "." << name
           << " = (Token){.type = TokenType::" << to_string(token.type);
    if (!token.value.empty()) {
      output << ", .value = \"" << token.value << "\"";
    }
    output << "},\n";
  }

  void field(const std::string &name, const std::vector<Token> &tokens) {
    if (tokens.size() > 0) {
      put_indent();
      output << "." << name << " = {\n";

      indent += 1;

      for (auto &token : tokens) {
        put_indent();
        output << " (Token){.type = TokenType::" << to_string(token.type);
        if (!token.value.empty()) {
          output << ", .value = \"" << token.value << "\"";
        }
        output << "},\n";
      }

      indent -= 1;

      put_indent();
      output << "},\n";
    } else {
      output << "." << name << " = {},\n";
    }
  }

  void put_indent() {
    for (int i = 0; i < indent; i++) {
      output << " ";
    }
  }

  Root root;

  std::stringstream output{};
  int indent = 0;
};
