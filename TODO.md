# Todo

## VM

- Closures
    - multiple levels deep
    - closing upvalues in loops (need to implement something like OP_CLOSE_UPVALUE)

- Mutate in-place instructions (faster arrays)

- Improve error handling in compiler, phases shouldn't need to error with an Exception

- `Module.field = "xyz"` assignment

- Compile time checking of `Module.field` syntax - ensure `.field` actually exists

- Less PushNil, Pop instructions


## Future

Higher priority:

- `match` statement, something like:
    ```
      match card {
        "A": 14,
        "K": 13,
        "Q": 12,
        "J": 11,
        "T": 10,
        _: int(card)
      }
    ```

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
- Pattern matching?
    - `match` statement
    - `[a, b, ...rest]`, `[...rest, y, z]`, `[a, b, ...rest, y, z]`
- UFCS? Is it even possible?
- Map merge syntax ala python: `{a: 1} | {a: 2, b: 3} == {a: 2, b: 3}`
- Builtins.reverse -> Enum.reverse

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
