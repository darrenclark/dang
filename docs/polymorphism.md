# Polymorphism

I want to write code like this:

```
// stdlib/Std/Range.dang
module Std/Range

let new = fn(start, end) {
    assert(start <= end)
    Range{start: start, end: end}
}

let iter = fn(range) {
    var i = 0
    fn () {
        if range.start + i < end { nil }
        else {
            i = i + 1
            range.start + i
        }
    }
}
```

```
// range_fun.dang
import Std/Range

// prints numbers 1-10 (or 1-9?)
for i in Range.new(1, 10) {
    println(i)
}

// prints 1, 4, 9
Range.new(1, 10) |> map(fn (x) { x * x }) |> inspect()
```

To do this:

- modules should be assignable to variables
- tagging dicts as being of a custom type
- `for _ in _`, `map(enum, func)`, etc. need a way to call functions on this type

## Modules in variables

Modules will be assignable to variables.  Dot (`.`) syntax will be used to access
the functions/fields defined in them:

```
module Greeter/Human

let greeting_phrase = fn (name) {
    "Hello, " + name
}
```

```
module Greeter/Dog

let greeting_phrase = fn (name) {
    "woof woof"
}
```

```
import Greeter/Human
import Greeter/Dog

let greet = fn(greeter, name) {
    println(greeter.greeting_phrase(name))
}

// would print "Hello, Bob"
greet(Human, "Bob")

// would print "woof woof"
greet(Dog, "Bob")
```

## Tagged Dicts

Inspired by Elixir's `defstruct` / `%{__struct__: Thing, ...}`, dictionaries
have a special key to denote that they are some special object. They'll be
called "tagged dicts".

Inspired by Objective-C, this field will be called `isa`:

```
module Std/Range

{isa: Range, start: 5, end: 10}
```

Additional syntax will be added to make this more ergonomical:

```
module Std/Range

Range{start: 5, end: 10)
```

## "Interfaces"

Initially, any "interfaces" will be implicit.  And calling methods on "interfaces" will
be done manually.  This will look something like this for `Enum`:

```
module Std/Enum

/// For something to be considered enumerable, it must either:
///
/// - a list, string, dict, or
/// - a tagged dict whose module implements `iter(self)`

/// Applies a function to each element and returns the result as a list
let map = fn (enumerable, func) {
    var result = []
    for item in iter(enumerable) {
        result = result + [func(item)]
    }
    result
}

/// Gets an iterator for the give object
let iter = fn (enumerable) {
    if type(enumerable) == 'list' {
        // lists are intrinsically iterable
        enumerable
    } else if type(enumerable) == 'string' {
        // strings are intrinsically iterable
        enumerable
    } else if type(enumerable) == 'dict' {
        if enumerable["isa"] != nil {
            // Use the tagged dict's module to create an iterator
            // (whatever it means for that type)
            enumerable["isa"].iter()
        } else {
            // Otherwise, create a bunch of key/value pairs (TODO: should make
            // dictionaries intrinsically iterable)
            enumerable |> map(fn (k) { {key: k, value: enumerable[k]} })
        }
    } else {
        raise("not an enumerable")
    }
}
```

### Language integration

For ergonomics sake, the language will automatically integrate with the
standard library where it makes sense.

Some ideas:

- for loops
    ```
    for x in y { ... }

    // would be interpreted as:

    for x in Enum.iter(y) { ... }
    ```

- `in`/contains operator (doesn't exist currently) might look like
    ```
    if x in y { ... }

    // would be interpreted as:

    import Std/SetAlgebra
    if SetAlgebra.contains?(y, x) { ... }
    ```

These would then call through to `isa` modules if necessary
