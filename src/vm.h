#pragma once

#include <iostream>
#include <optional>
enum Op : int {
  // const_int  X :  pushes integer X
  const_int,
  // get_local  N :  Gets local variable (N is relative to frame pointer)
  get_local,
  // multiply     :  Multiplies top to elements on stack
  multiply,
  // return       :  Returns top value on stack
  return_
};

class VM {
public:
  VM(int *code)
      : code(code), ip(code), stack(new int[1024]), fp(stack), sp(stack) {}
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
    case Op::const_int:
      push(read_arg());
      trace("const_int  ");
      break;
    case Op::get_local:
      push(*(fp + read_arg()));
      trace("get_local  ");
      break;
    case Op::multiply: {
      int a = pop();
      int b = pop();
      push(a * b);
      trace("multiply   ");
      break;
    }
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

    std::cerr << std::endl;
  }

  int *code;
  int *ip;

  int *stack;
  int *fp, *sp;
};
