# Metafunctions

Structs can provide custom behaviours to some language features by implementing
"metafunctions".

## `__iter__(struct)`

Should return an iterator for the given struct.

Used by `for` loops and the built in `iter(iterable)` function
