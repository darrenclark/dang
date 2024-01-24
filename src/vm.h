#pragma once

#include <iostream>
#include <optional>

enum Op : int {
  // const_int  X :  pushes integer X
  const_int,
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
  case const_int:
    return "const_int";
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
  case const_int:
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

class VM {
public:
  VM(const int *code)
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

  const int *code;
  const int *ip;

  int *stack;
  int *fp, *sp;
};
