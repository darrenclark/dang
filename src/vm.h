#pragma once

#include "compiler.h"
#include <iostream>
#include <optional>

class VM {
public:
  VM(Chunk chunk)
      : chunk(chunk), code(this->chunk.code.data()), ip(code),
        stack(new int[1024]), fp(stack), sp(stack) {}
  ~VM() { delete[] stack; }

  int run() {
    while (true) {
      if (auto res = step()) {
        return *res;
      }
    }
  }

private:
  std::optional<int> step() {
    std::optional<int> result = std::nullopt;

    switch (*ip++) {
    case Op::load_const:
      push(chunk.constants.at(read_arg()));
      trace("load_const  ");
      break;
    case Op::get_local:
      push(*(fp + read_arg()));
      trace("get_local  ");
      break;
    case Op::set_local: {
      *(fp + read_arg()) = pop();
      trace("set_local  ");
      break;
    }
    case Op::add: {
      int a = pop();
      int b = pop();
      push(a + b);
      trace("add   ");
      break;
    }
    case Op::subtract: {
      int a = pop();
      int b = pop();
      push(b - a);
      trace("add   ");
      break;
    }
    case Op::multiply: {
      int a = pop();
      int b = pop();
      push(a * b);
      trace("multiply   ");
      break;
    }
    case Op::divide: {
      int a = pop();
      int b = pop();
      push(b / a);
      trace("multiply   ");
      break;
    }
    case Op::pop:
      pop();
      trace("pop   ");
    case Op::return_:
      result = pop();
      trace("return     ");
    }

    return result;
  }

  int read_arg() { return *ip++; }

  void push(int value) {
    *sp = value;
    sp++;
  }

  int pop() { return *--sp; }

  void trace(const char *op) {
    std::cerr << op << "   stack:";

    for (int *i = stack; i < sp; i++) {
      std::cerr << " " << *i;
    }

    std::cerr << "  [ip: " << (ip - code) << "]";

    std::cerr << std::endl;
  }

  const Chunk chunk;

  const int *code;
  const int *ip;

  int *stack;
  int *fp, *sp;
};
