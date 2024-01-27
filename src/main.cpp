#include "ast_printer.h"
#include "compiler.h"
#include "disassembler.h"
#include "vm.h"
#include <fstream>
#include <sstream>

static std::string read_program(const char *path) {
  if (std::string(path) == "-") {
    std::stringstream s;
    s << std::cin.rdbuf();
    return s.str();
  } else {
    std::ifstream f(path);
    if (!f) {
      std::cerr << "failed to open file: " << path << std::endl;
      exit(EXIT_FAILURE);
    }

    std::stringstream s;
    s << f.rdbuf();
    return s.str();
  }
}

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cerr << "error: invalid arguments" << std::endl;
    std::cerr << "usage: " << argv[0] << " path/to/program.dang" << std::endl;
    exit(EXIT_FAILURE);
  }

  std::string source = read_program(argv[1]);

  VM vm;
  Value result = vm.eval(source);

  std::cout << result.to_string() << std::endl;
}

/*int main() {

  // clang-format off
  int program[] = {
    Op::const_int, 3,
    Op::const_int, 100,
    Op::get_local, 1,
    Op::get_local, 1,
    Op::multiply,
    Op::get_local, 0,
    Op::multiply,
    Op::return_
  };
  // clang-format on

  Disassembler disasm;
  std::cerr << disasm.disassemble(program);

  std::cerr << std::endl;

  VM vm(program);
  int result = vm.run();

  std::cerr << std::endl;
  std::cout << result << std::endl;
}*/
