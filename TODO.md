# Todo

## VM

- Mutate in-place instructions (faster arrays)

- Improve error handling in compiler, phases shouldn't need to error with an Exception

- Compile time checking of `Module.field` syntax - ensure `.field` actually exists

- Less PushNil, Pop instructions


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

Lower priority:

- Bitwise operators / functions
- big int
- more type casting
    - "try" casts
- Regex captures
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
