#pragma once

#include "vm.h"
#include <span>
#include <sstream>

class Disassembler {
public:
  std::string disassemble(std::span<const int> code) {
    std::stringstream out;
    size_t offset = 0;

    while (offset < code.size()) {
      Op op_code = (Op)code[offset];
      std::string op_code_str = to_string(op_code);
      out << op_code_str;
      for (int i = op_code_str.size(); i < OP_CODE_COLUMN_WIDTH; i++)
        out << " ";

      offset++;

      for (int i = 0; i < op_n_args(op_code); i++) {
        if (offset >= code.size()) {
          out << "[ERROR: End of code, expected arguments]";
          return out.str();
        }

        out << " " << code[offset];
        offset++;
      }

      out << std::endl;
    }

    return out.str();
  }

private:
  static const int OP_CODE_COLUMN_WIDTH = 10;
};
