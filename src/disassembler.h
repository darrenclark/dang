#pragma once

#include "compiler.h"
#include <span>
#include <sstream>

class Disassembler {
public:
  std::string disassemble(Function function) {
    out = std::stringstream();

    print_function(function);

    return out.str();
  }

private:
  void print_function(Function function) {
    out << "== " << function.name << " ==\n";

    print_code(*function.chunk);

    for (const Value &v : function.chunk->constants) {
      if (v.type() == ValueType::function) {
        print_function(v.function_value());
      }
    }
  }

  void print_code(const Chunk &chunk) {
    size_t offset = 0;

    while (offset < chunk.code.size()) {
      Op op_code = (Op)chunk.code[offset];
      std::string op_code_str = to_string(op_code);
      out << op_code_str;
      for (int i = op_code_str.size(); i < OP_CODE_COLUMN_WIDTH; i++)
        out << " ";

      offset++;

      for (int i = 0; i < op_n_args(op_code); i++) {
        if (offset >= chunk.code.size()) {
          out << "[ERROR: End of code, expected arguments]";
          out << std::endl << std::endl;
          return;
        }

        out << " " << chunk.code[offset];
        offset++;
      }

      out << std::endl;
    }

    out << std::endl << std::endl;
  }

private:
  std::stringstream out;
  static const int OP_CODE_COLUMN_WIDTH = 14;
};
