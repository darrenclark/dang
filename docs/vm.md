# VM

## Examples

```dang
let add = fn(a, b) {
    a + b
}

print(add(14, 32))
```

Would emit:

```
==== add (const@0) =====

GET_LOCAL    1
GET_LOCAL    2
ADD
RETURN

==== __main__ ====

// prep for print(...)
CONSTANT    3  // <Std/Builtins>
CONSTANT    4  // <print>
GET_GLOBAL

// add(14, 32)
CONSTANT    0  // <add(..)>
CONSTANT    1  // <14>
CONSTANT    2  // <34>
CALL        2  // add(14, 32)

// call print(...)
CALL        1
RETURN
```

## Op codes

- `CONSTANT` - load the constant value at index I
- `ADD` `SUB` `MUL` `DIV` - math on top 2 values

- `GET_GLOBAL` - uses top 2 values (module, name) to get a global
- `SET_GLOBAL` - uses top 3 values (module, name, NewValue) to update a global
- `GET_LOCAL` - gets local value relative to frame pointer
- `SET_LOCAL` - sets local value relative to frame pointer

- `CALL` call function
- `RETURN` return from a function

## To compile `Std/Builtins`

- `__builtin print` compiles to:
    ```
    CONSTANT 0  // Symbol(Std/Builtins)
    CONSTANT 1  // Symbol(print)
    CONSTANT 2  // NativeFunc(print)
    SET_GLOBAL
    ```

- `let INT32_MAX = 2147483647` compiles to:
    ```
    CONSTANT 0  // Symbol(Std/Builtins)
    CONSTANT 3  // Symbol(INT32_MAX)
    CONSTANT 4  // Int(2147483647)
    SET_GLOBAL
    ```

- `let some = symbol("some")` compiles to:
    ```
    CONSTANT 0  // Symbol(Std/Builtins)
    CONSTANT 5  // Symbol(some)

    // symbol("some")
    CONSTANT 0  // Symbol(Std/Builtins)
    CONSTANT 6  // Symbol(symbol)
    GET_GLOBAL
    CONSTANT 7  // "some"
    CALL

    SET_GLOBAL
    ```
