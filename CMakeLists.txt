cmake_minimum_required(VERSION 3.28)

find_package(Catch2 3 REQUIRED)

project(dang VERSION 1.0 LANGUAGES CXX)

# Main
add_executable(dang src/main.cpp)
set_property(TARGET dang PROPERTY CXX_STANDARD 20)

# Tests
list(APPEND CMAKE_CTEST_ARGUMENTS "--output-on-failure")

add_executable(tests test/example.cpp src/calc.cpp test/lexer_test.cpp)

target_link_libraries(tests PRIVATE Catch2::Catch2WithMain)
set_property(TARGET tests PROPERTY CXX_STANDARD 20)

include(CTest)
include(Catch)
catch_discover_tests(tests)
