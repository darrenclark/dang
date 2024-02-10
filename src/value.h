#include <iostream>
#include <memory>
#include <string>
#include <variant>

enum class ValueType { int_, double_, string, function };

inline std::string to_string(ValueType t) {
  switch (t) {
  case ValueType::int_:
    return "int";
  case ValueType::double_:
    return "double";
  case ValueType::string:
    return "string";
  case ValueType::function:
    return "function";
  }
}

struct Chunk;

struct Function {
  std::string name;
  int arity;
  std::shared_ptr<Chunk> chunk;

  bool operator==(const Function &) const = default;
};

struct Value {
  std::variant<int, double, std::string, Function> value;

  ValueType type() const {
    if (std::holds_alternative<int>(value)) {
      return ValueType::int_;
    } else if (std::holds_alternative<double>(value)) {
      return ValueType::double_;
    } else if (std::holds_alternative<std::string>(value)) {
      return ValueType::string;
    } else if (std::holds_alternative<Function>(value)) {
      return ValueType::function;
    } else {
      assert(false);
    }
  }

  bool is_numeric() const {
    switch (type()) {
    case ValueType::int_:
    case ValueType::double_:
      return true;
    default:
      return false;
    }
  }

  double double_value() const {
    if (const int *v = std::get_if<int>(&value)) {
      return (double)*v;
    } else {
      return std::get<double>(value);
    }
  }

  int int_value() const { return std::get<int>(value); }

  std::string string_value() const { return std::get<std::string>(value); }

  Function function_value() const { return std::get<Function>(value); }

  static Value of(int v) { return Value{.value = v}; }

  static Value of(double v) { return Value{.value = v}; }

  static Value of(const std::string &v) { return Value{.value = v}; }

  bool operator==(const Value &) const = default;

  std::string to_string() const {
    struct ToStringVisitor {
      std::string operator()(int v) const { return std::to_string(v); }
      std::string operator()(double v) const { return std::to_string(v); }
      std::string operator()(const std::string &v) const { return v; }
      std::string operator()(const Function &v) const {
        return "#<Function(" + v.name + ")>;";
      }
    };
    return std::visit(ToStringVisitor{}, value);
  }

  operator bool() const {
    struct BoolVisitor {
      bool operator()(int v) const { return v != 0; }
      bool operator()(double v) const { return v != 0.0; }
      bool operator()(const std::string &v) const { return v.size() > 0; }
      bool operator()(const Function &v) const { return true; }
    };
    return std::visit(BoolVisitor{}, value);
  }

  Value &operator+=(const Value &rhs) {
    if (type() == ValueType::int_ && rhs.type() == ValueType::int_) {
      *this = Value{.value = int_value() + rhs.int_value()};
    } else if (is_numeric() && rhs.is_numeric()) {
      *this = Value{.value = double_value() + rhs.double_value()};
    } else if (type() == ValueType::string && rhs.type() == ValueType::string) {
      *this = Value{.value = string_value() + rhs.string_value()};
    } else {
      invalid_operands_error(rhs, "+");
    }

    return *this;
  }

  friend Value operator+(Value lhs, const Value &rhs) {
    lhs += rhs;
    return lhs;
  }

  Value &operator-=(const Value &rhs) {
    if (type() == ValueType::int_ && rhs.type() == ValueType::int_) {
      *this = Value{.value = int_value() - rhs.int_value()};
    } else if (is_numeric() && rhs.is_numeric()) {
      *this = Value{.value = double_value() - rhs.double_value()};
    } else {
      invalid_operands_error(rhs, "-");
    }
    return *this;
  }

  friend Value operator-(Value lhs, const Value &rhs) {
    lhs -= rhs;
    return lhs;
  }

  Value &operator*=(const Value &rhs) {
    if (type() == ValueType::int_ && rhs.type() == ValueType::int_) {
      *this = Value{.value = int_value() * rhs.int_value()};
    } else if (is_numeric() && rhs.is_numeric()) {
      *this = Value{.value = double_value() * rhs.double_value()};
    } else {
      invalid_operands_error(rhs, "*");
    }
    return *this;
  }

  friend Value operator*(Value lhs, const Value &rhs) {
    lhs *= rhs;
    return lhs;
  }

  Value &operator/=(const Value &rhs) {
    if (type() == ValueType::int_ && rhs.type() == ValueType::int_) {
      *this = Value{.value = int_value() / rhs.int_value()};
    } else if (is_numeric() && rhs.is_numeric()) {
      *this = Value{.value = double_value() / rhs.double_value()};
    } else {
      invalid_operands_error(rhs, "/");
    }
    return *this;
  }

  friend Value operator/(Value lhs, const Value &rhs) {
    lhs /= rhs;
    return lhs;
  }

private:
  [[noreturn]] void invalid_operands_error(const Value &rhs, const char *op) {
    std::cerr << "invalid operands to " << op << ": " << ::to_string(type())
              << " and " << ::to_string(rhs.type());
    exit(EXIT_FAILURE);
  }
};

inline std::ostream &operator<<(std::ostream &os, Value const &value) {
  os << value.to_string();
  return os;
}
