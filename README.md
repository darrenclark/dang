# dang

Hobby programming language.  Aiming to have a language functional enough to do Advent of Code 2024 🤞

*Inspired by Pixeled's "[Creating a Compiler](https://www.youtube.com/playlist?list=PLUDlas_Zy_qC7c5tCgTMYq2idyyT241qs)" (https://github.com/orosmatthew/hydrogen-cpp)*

## Building

```sh
cmake -B./build -DCMAKE_EXPORT_COMPILE_COMMANDS=1 .
cd build
make

# to run tests
ctest
```
