# Pattern Matching

Support pattern matching in a:

- a `match` statement, i.e.:

```
  match card {
    "A" -> 14
    "K" -> 13
    "Q" -> 12
    "J" -> 11
    "T" -> 10
    _ -> int(card)
  }
```

- `let` / `var` / assignment (beyond existing tuple)

## Match statements

- Match statements should support:
    - matching against constants, i.e. `"some string"`, `1`, `true`, `:symbol`, `nil`, etc.
    - matching against tuples, i.e. `(a, b)`
    - matching against lists:
        - `[a, ..b]` would give `a` as a single value, and `b` as a list of remaining elements
        - The special `..b` could be placed at most once, anywhere in the match
        - Syntax wise `..b`, `b..`, `..b..` will all be the same - "match remaining elements".
        - `[]` would match an empty list
        - `[..]` would match any list
    - matching against strings - `"hello " + person + "!"`
        - `+ person +` would behave just like the `..b` above - only one instance per match
        - `"" + _ + ""` (or `_ + ""` / `"" + _`) would match any string
    - matching against dictionaries:
        - `{key: value}` would match a dictionary that has `key` (irregardless of other keys)
        - `{"key" => value}` would also work
        - `{}` would match any map
    - matching against structs:
        - `StructName{key: value}` will match `StructName` instances
        - unlike Elixir, will require `StructName` always resolves to a struct (can't put a placeholder to extract the name of it)
        - A special `_{}` syntax will be used to match "any struct"

    - assigning names to matched terms. Potential options:
        `a @ _{} ->` - "a is a struct"
        `a = _{} ->` - "a is a struct" - possibly parser confusion with assigment
        `_{} = a ->` - "a is a struct" - possibly parser confusion with assigment
        `a is _{} ->` - "a is a struct" - could be nice if `is` is used for type checking, ie.:
            ```
            if a is int {
            ```

    - additional conditions per branch using `if` syntax, i.e.:
        ```
        match age {
            a if is_int(a) && a >= 18 -> "old enough to vote"
            a if is_str(a) && int(a) >= 18 -> "old enough to vote"
            _ -> "wait a little longer"
        }
        ```

    - either single statements or blocks
        ```
        match list {
            [] -> "empty list"
            [..] -> {
                var msg = str(len(list))
                msg = msg + " items in list"
                msg
            }
        }
        ```

## Implementation

Similar to constants, `Chunk` would contain a list of `MatchPattern`

`MatchPattern` would be a Rust struct (closely mimicking the AST) that the VM can use to evaluate a match.

The VM will have new instructions:

- `Match <P>` executes a match (using pattern P) and puts the named match results on the stack

- `TryMatch <P> <N:16bit>` attempts a match (using pattern P) and either puts the named match results on the stack or jumps `N` instructions ahead

- `MatchFailure <P1> <P2>` - raise an error saying the value at the top of the stack didn't match any pattern in the patterns at indexes P1 - P2 inclusive

### Assignment

Assignment (including `let` and `var`) will use the `Match` instruction

For `let`/`var` of locals - all named values will be in correct stack slots

For assignment, or `let`/`var` of globals - compiler will emit `SetLocal`, `SetGlobal`, etc. instructions

### `match`

- `TryMatch` for each case
    - on success:
        - if applicable, run through the `if` clause.  If it fails, pop all local values and jump to the next case, otherwise pop the value and continue on to the body
    - on failure jump to the next case or (if the last case), jump to a `MatchFailure` instruction

