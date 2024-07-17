# Todo

Priorities:

1. finish imports
2. structs
3. tuples
4. iterators

## Tuples

```
()      // empty tuple
(5,)    // single element tuple (disambiguate with parenthesized expression)
(5,5)   // two element tuple
(5,5,)  // two element tuple with the optional trailing comma

let tuple = (5,1,)
assert(len(tuple) == 2)
assert(tuple[0] == 5)
assert(tuple[1] == 1)

// destructuring
let (a, b) = tuple

// iterating over dictionary
let dict = {a: 1, b: 2, c: 3}
for (k, v) in dict {
    println(k + " = " + str(v))
}

// iterating with index
for (i, v) in enumerate(["a", "b", "c"]) {
    println(str(i) " = " + v)
}

// permutations
let items_a = [1, 2, 3]
let items_b = ["a", "b"]
for (a, b) in permutations(items_a, items_b) {
    println(str(a) "," + b)
}
```

## Modules

- imports:
    - `import Std/Enum.*` - bring in all
    - `import Std/Enum.map` - bring in single
    - `import Std/Enum` - set `Enum` to `symbol("Std/Enum")`
    - `import Std/Enum as StdEnum` - set `StdEnum` to `symbol("Std/Enum")`
- structs:
    ```
    module Std/Point { x: 0, y: 0 }
    let pt1 = @Point{y: 2}
    let pt2 = @Point{x: 1, y: 2}
    ```
- structs implementing enumerable:
    - `for i in @Range{start: 0, end: 10} { println(i) }`
- get module for struct:
    ```
    let pt1 = @Point{}
    assert(pt1@ == Point)
    ```

- inline modules:
    ```
    import {
        module Point {x: 0, y: 0}

        let new = fn (x, y) { @Point{x: x, y: y} }

        let add = fn (a, b) {
            @Point{x: a.x + b.x, y: a.x + b.x}
        }
    }
    ```

## Future

- Maps
    - Symbols/atoms for keys?
- Structs? - or should everything be a map?
- Tuples?
- Bitwise operators / functions
- big int
- Iterators
- more type casting
    - "try" casts
- Regex captures
- descriptive assert (show different parts of expression)
- Shadowing library provided functions (perhaps it gives a warning instead?)
- Pattern matching?
