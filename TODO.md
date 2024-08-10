# Todo

## VM

- Validate function arity

- Closures
    - multiple levels deep
    - closing upvalues in loops (need to implement something like OP_CLOSE_UPVALUE)

- Mutate in-place instructions (faster arrays)

- disassembly shouldn't print duplicate functions
    - and should support filtering to a module (by default the main script)

- Debug info in compiled chunks
    - line numbers per instruction
    - function names
    - exception stack traces
    - variable names (?)

- Improve error handling in compiler, phases shouldn't need to error with an Exception

- `Module.field = "xyz"` assignment

## Future

Higher priority:

- `fn name(...) { ... }` syntax equivalent to `let name = fn(..) { ... }`
- Module references - i.e. `@Std/Enum.map` should work without import
- metamethods for structs:
    - `__new__(...)` for `Point(...)` syntax
    - `__iter__(s)` for iterators
    - `__inspect__(s)` or `__to_str__(s)` for printing
    - `__subscript__(..)` for subscripting
    - `__add__(a, b)` for `+`
    - etc.
- Lightweight way to write tests - maybe `test "some test case" {` blocks?

- `_` pattern matching
- `zip` function

Lower priority:

- Maps - Symbols/atoms for keys?
- Bitwise operators / functions
- big int
- more type casting
    - "try" casts
- Regex captures
- descriptive assert (show different parts of expression)
- Shadowing library provided functions (perhaps it gives a warning instead?)
- Pattern matching?
    - `match` statement
    - `[a, b, ...rest]`, `[...rest, y, z]`, `[a, b, ...rest, y, z]`
- UFCS? Is it even possible?
- Map merge syntax ala python: `{a: 1} | {a: 2, b: 3} == {a: 2, b: 3}`

## Bugs / Limitations

- Bad error message when escape code is invalid, i.e. `"\ghi"`
- Vim != highlight doesn't work
- Can't assign to vars in other modules, i.e.:
    ```
    import SomeModule
    SomeModule.y = 5
    ```
- Can't assign to chars within a string, i.e.:
    ```
    var s = "123"
    s[1] = "9"
    ```
