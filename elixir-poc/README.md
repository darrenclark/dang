# Dang

To run:

```
mix escript.build && ./dang
```

## Examples

```
# if statement
(let x 5)
(if (== x 5) (do
  (let y 12)
  (let z (list x y))
  (print z)
) else (do
  (print "not five")
))

# iterate over list
(let items (list 1 2 3 4 5))
(var sum 0)
(for x items (do
  (set sum (+ sum x))
))
(print "Sum: " sum)
```

## TODO

- REPL
- Comparison short circuiting
- Ensure `set` only mutates `var`s, not `let`
- Ensure scoping works correctly with if & loops
- Better enum interface (make it easier to implement map, filter ,etc.)
