#include <catch2/catch_test_macros.hpp>

#include "../src/calc.h"

TEST_CASE( "Addition works", "[example]" ) {
  REQUIRE( add(5, 3) == 8 );
}
