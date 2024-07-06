# Modules

```
// Specifies this file is a module
module Std/Set

// new and contains? will be exported
let new = fn(elements) {
    var dict = {}
    for e in elements { dict[e] = true }

    // "Set" will be usable, just as if it had been imported
    // (see ./polymorphism.md)
    Set{elements: dict}
}

let contains? = fn(set, element) {
    set.elements[element] != nil
}
```

```
// Loads the Std/Set module
import Std/Set

// Fields are accessible under `Set`:
let s = Set.new(["a", "b", "c"])
Set.contains?(s, "d")
```

```
// Loads the Std/Set module, but only imports a couple functions.
import Std/Set.new
import Std/Set.contains?

// Functions are accessible without any qualification:
let s = new(["a", "b", "c"])
contains?(s, "d")
```

```
// Loads the Std/Set module, and imports all functions/data/etc.
import Std/Set.*

// Functions are accessible without any qualification:
let s = new(["a", "b", "c"])
contains?(s, "d")
```

**FUTURE EVOLUTION**:

- `import Std/Set as S` syntax for aliasing
- `module Example { ... }` syntax for defining modules in line

## Constraints

- All `let` and `var` will be exported (no public/private/etc visibiltiy modifiers)
- No circular imports
    - Great simplifies startup
- An `import` is required to access those functions
    - ie. `Std/Set.new(["a", "b", "c"])` will be considered invalid (hard/impossible to disambiguate with division operator)

## Implicit imports

Each "user" `.dang` file will have some implicit imports for ergonomics:

```
import Std/Enum.*
```

This will not apply to stdlib (as this would cause circular imports)

## Resolving imports to file paths

### From "main" file

```
import Std/Set

let s = Set.new([1, 2, 3])
inspect(s)
```

As the main file (the script) won't have a `module`, apply these rules:

Loads the file by searching "DANG_MODULE_SEARCH_PATHS" for `Std/Set.dang`

Default `DANG_MODULE_SEARCH_PATHS` is `"(stdlib)/:(projectroot)/"`

For single scripts (i.e. `abc.dang`), the `(projectroot)` is the folder the
script is in.

**FUTURE EVOLUTION**: `dang.toml` file that signals project root?

### Modules relative to each other

Because these have the same prefix (`Lib/`), this file (`Abc.dang`) will
attempt to find `Lib/Def` at `Def.dang` in the same folder.

```
module Lib/Abc

import Lib/Def
```

This also applies for "Root" modules:

```
module Abc

import Def
```
