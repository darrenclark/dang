#include "interpreter.h"
#include "lexer.h"
#include "parser.h"
#include <fstream>
#include <sstream>

static std::string read_program(const char *path) {
  std::fstream f(path, std::ios::in);
  std::stringstream s;
  s << f.rdbuf();
  return s.str();
}

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cerr << "error: invalid arguments" << std::endl;
    std::cerr << "usage: " << argv[0] << " path/to/program.dang" << std::endl;
  }

  std::string source = read_program(argv[1]);

  Lexer lexer(source);
  auto tokens = lexer.lex();

  Parser parser(tokens);
  auto ast = parser.parse();

  Interpreter interpreter(ast);

  int result = interpreter.run();

  std::cout << result << std::endl;
}
