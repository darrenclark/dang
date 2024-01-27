#pragma once

#include "compiler.h"
#include "disassembler.h"
#include <iostream>
#include <optional>

#define DISASSEMBLE 1

class VM {
public:
  VM() : stack(new Value[1024]), fp(stack), sp(stack) {}
  ~VM() { delete[] stack; }

  Value eval(const std::string &source) {
    Compiler compiler;
    chunk = compiler.compile(source);
    code = chunk.code.data();
    ip = code;

#if DISASSEMBLE
    Disassembler d;
    std::cerr << d.disassemble(chunk) << std::endl;
#endif

    while (true) {
      if (auto res = step()) {
        return *res;
      }
    }
  }

private:
  std::optional<Value> step() {
    std::optional<Value> result = std::nullopt;

    switch ((Op)*ip++) {
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
      Value a = pop();
      Value b = pop();
      push(a + b);
      trace("add   ");
      break;
    }
    case Op::subtract: {
      Value a = pop();
      Value b = pop();
      push(b - a);
      trace("subtract   ");
      break;
    }
    case Op::multiply: {
      Value a = pop();
      Value b = pop();
      push(a * b);
      trace("multiply   ");
      break;
    }
    case Op::divide: {
      Value a = pop();
      Value b = pop();
      push(b / a);
      trace("divide   ");
      break;
    }
    case Op::pop:
      pop();
      trace("pop   ");
      break;
    case Op::jump: {
      int n = read_arg();
      ip += n;
      trace("jump  ");
      break;
    }
    case Op::jump_if_zero: {
      int n = read_arg();
      ip += pop() ? 0 : n;
      trace("jump_if_zero  ");
      break;
    }
    case Op::return_:
      result = pop();
      trace("return     ");
      break;
    case Op::OP_COUNT:
      assert(false);
    }

    return result;
  }

  int read_arg() { return *ip++; }

  void push(Value value) {
    *sp = value;
    sp++;
  }

  Value pop() { return *--sp; }

  void trace(const char *op) {
    std::cerr << op << "   stack:";

    for (Value *i = stack; i < sp; i++) {
      std::cerr << " " << i->to_string();
    }

    std::cerr << "  [ip: " << (ip - code) << "]";

    std::cerr << std::endl;
  }

  Chunk chunk;

  const int *code;
  const int *ip;

  Value *stack;
  Value *fp, *sp;
};
