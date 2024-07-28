# Todo

Priorities:

1. ~modules refactor~ (mostly done)
2. structs
3. ~tuples~
4. iterators

## Future

Higher priority:

- `fn name(...) { ... }` syntax equivalent to `let name = fn(..) { ... }`
- Module references - i.e. `@Std/Enum.map` should work without import
- Better inspect output/Values printed as valid source code
- metamethods for structs:
    - `__new__(...)` for `Point(...)` syntax
    - `__iter__(s)` for iterators
    - `__inspect__(s)` or `__to_str__(s)` for printing
    - `__subscript__(..)` for subscripting
    - `__add__(a, b)` for `+`
    - etc.
- Lightweight way to write tests - maybe `test "some test case" {` blocks?

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
- UFCS? Is it even possible?

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
