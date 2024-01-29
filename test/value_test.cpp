#include <catch2/catch_test_macros.hpp>

#include "../src/value.h"

TEST_CASE("adding strings works correctly", "[value]") {
  Value res = Value::of("Hello, ") + Value::of("world");

  REQUIRE(res == Value::of("Hello, world"));
}
