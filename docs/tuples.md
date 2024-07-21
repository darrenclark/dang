# Tuples

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

## Why tuples? Why not only lists?

- Technically speaking - tuple & list will likely be implemented very similar

- Slightly nicer ergonomics.
    ```
    // points as tuples
    [(1, 5), (0, 4), (2, 1)]
    // vs. points as lists
    [[1, 5], [0, 4], [2, 1]]

    // return tuple
    let get_color_and_count = fn () {
        ("red", 5)
    }
    // vs returning list
    let get_color_and_count = fn () {
        ["red", 5]
    }
    ```

- Better opportunities for type checking
    ```
    // this would be `[Int | String]` ("list where each value is either a string or int")
    ["red", 5]
    // vs. `(String, Int)` ("tuple, first element is string, second is int")
    ("red", 5)
    ```

## Implementation

### Destructuring

Starting out with:

```
let (x, y) = (1, 2)
```

But will want to eventually support something like:

```
match (1, 2) {
    (1, _) -> 1
    (2, 2) -> 0
    (_, _) -> 0
}
```

To implement, perhaps the code looks like

```
//pseudo code

let pattern = ...;
let rhs = self.eval(ast)
let vars_and_vals = self.pattern_match(pattern, rhs)

for (var, val) in vars_and_vals {
    self.define(var, val)
}
```

The AST for `let (x, y) = `should look something like:

```
Let
    Pattern:
        TuplePattern:
            IdentifierPattern
            IdentifierPattern
    Expr: ...
```
