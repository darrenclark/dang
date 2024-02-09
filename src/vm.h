#pragma once

#include "compiler.h"
#include "disassembler.h"
#include <iostream>
#include <optional>

#define DISASSEMBLE 0
#define TRACE 0

struct Frame {
  Function function;
  int *ip;
  Value *fp; // correct name?
};

class VM {
public:
  VM() : stack(new Value[1024]), sp(stack) {}
  ~VM() { delete[] stack; }

  Value eval(const std::string &source) {
    sp = stack;

    Compiler compiler;
    Function function = compiler.compile(source);
    enter_function(function);

#if DISASSEMBLE
    Disassembler d;
    std::cerr << d.disassemble(current_chunk()) << std::endl;
#endif

    while (true) {
      if (auto res = step()) {
        return *res;
      }
    }
  }

private:
  void enter_function(const Function &function) {
    frames.push_back(Frame{
        .function = function, .ip = function.chunk->code.data(), .fp = sp});
  }

  std::optional<Value> step() {
    std::optional<Value> result = std::nullopt;

    switch ((Op)*current_frame().ip++) {
    case Op::load_const:
      push(current_chunk().constants.at(read_arg()));
      trace("load_const  ");
      break;
    case Op::define_global: {
      std::string name =
          current_chunk().constants.at(read_arg()).string_value();
      auto it = globals.find(name);
      if (it != globals.end()) {
        std::cerr << "global '" << name << "' already defined" << std::endl;
        exit(EXIT_FAILURE);
      }
      globals[name] = pop();
      trace("define_global  ");
      break;
    }
    case Op::get_global: {
      std::string name =
          current_chunk().constants.at(read_arg()).string_value();
      auto it = globals.find(name);
      if (it == globals.end()) {
        std::cerr << "global '" << name << "' not defined" << std::endl;
        exit(EXIT_FAILURE);
      }
      push(it->second);
      trace("get_global  ");
      break;
    }
    case Op::set_global: {
      std::string name =
          current_chunk().constants.at(read_arg()).string_value();
      auto it = globals.find(name);
      if (it == globals.end()) {
        std::cerr << "global '" << name << "' not defined" << std::endl;
        exit(EXIT_FAILURE);
      }
      it->second = pop();
      trace("set_global  ");
      break;
    }
    case Op::get_local:
      push(*(current_frame().fp + read_arg()));
      trace("get_local  ");
      break;
    case Op::set_local: {
      *(current_frame().fp + read_arg()) = pop();
      trace("set_local  ");
      break;
    }
    case Op::add: {
      Value a = pop();
      Value b = pop();
      push(b + a);
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
      push(b * a);
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
      current_frame().ip += n;
      trace("jump  ");
      break;
    }
    case Op::jump_if_zero: {
      int n = read_arg();
      current_frame().ip += pop() ? 0 : n;
      trace("jump_if_zero  ");
      break;
    }
    case Op::return_: {
      Value r = pop();
      sp = current_frame().fp;
      frames.pop_back();
      if (frames.size() == 0) {
        result = r;
      } else {
        push(r);
      }
      trace("return     ");
      break;
    }
    case Op::OP_COUNT:
      assert(false);
    }

    return result;
  }

  int read_arg() { return *current_frame().ip++; }

  void push(Value value) {
    *sp = value;
    sp++;
  }

  Value pop() { return *--sp; }

  void trace(const char *op) {
#if TRACE
    std::cerr << op << "   stack:";

    for (Value *i = stack; i < sp; i++) {
      std::cerr << " " << i->to_string();
    }

    if (frames.size() > 0) {
      std::cerr << "  [ip: "
                << (current_frame().ip - current_chunk().code.data()) << "]";
    } else {
      std::cerr << "  [ip: null]";
    }

    std::cerr << std::endl;
#endif
  }

  std::vector<Frame> frames;

  Value *stack;
  Value *sp;

  Frame &current_frame() { return frames.back(); }
  const Chunk &current_chunk() { return *current_frame().function.chunk; }

  std::unordered_map<std::string, Value> globals;
};
