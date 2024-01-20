#pragma once

#include "parser.h"
#include <string>

class Interpreter {
public:
  Interpreter(ASTNodeProgram program) : program(std::move(program)) {}

  int run() { return std::stoi(program.body.integer_literal.value); }

private:
  ASTNodeProgram program;
};
