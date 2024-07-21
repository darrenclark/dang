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

```
// references to modules are Symbols
import Std/Set

asssert(Set == symbol("Std/Set"))

let s = symbol("Std/Set").new(["a", "b", "c"])
```

**FUTURE EVOLUTION**:

- `import Std/Set as S` syntax for aliasing
- Syntax for defining modules in line
    - `module Example { ... }` 
    - ```
      import {
          module Point {x: 0, y: 0}

          let new = fn (x, y) { @Point{x: x, y: y} }

          let add = fn (a, b) {
              @Point{x: a.x + b.x, y: a.x + b.x}
          }
      }
      ```

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

## Refactor

Current issues:

- imports/exports system too rigid
    - everything is exported (even if it's a symbol we only really want visible internally)
    - no concept of variables vs constants (prevents inlining)
- every variable needs to be associated with a NodeId
    - makes it awkward at times (`insert_prelude_phase`)
    - (kinda) loses original variable names - debugging is harder
- ModuleId:
    - Good: lightweight - just an uint32 everywhere
    - Awkward: having it encoded in each NodeId - Compiler has to predict next ModuleId and use that

### Improvements

- Use interned strings (`ustr` crate?) instead of:
    - ModuleId
    - NodeId in Environment

- Update variable lookup to be something like this:
    ```rust
    enum VariableLocation {
        // local variable
        Local { name: Ustr },
        // closed over variable in nth parent environment (lexically)
        Closure { name: Ustr, nth_parent: usize },
        // global variable
        Global { module: Ustr, name: Ustr },
    }
    ```

- In interpreter, environments will something like:
    ```rust
    struct GlobalEnvironment {
        // (module, name) => value
        variables: HashMap<(Ustr, UStr), Value>,
    }

    struct LocalEnvironment {
        // pointer to parent
        parent: Option<Rc<RefCell<LocalEnvironment>>>,
        // (module, name) => value
        variables: HashMap<Ustr, Value>,
    }
    ```

- Code at rool level will leverage JS style variable hoisting.  An exception will be raised if a variable is "used" (code executed) before the variable has been initiated (may be a compiler error eventually)
    ```
    let part1 = fn(input) {
        filter(input, is_valid)
    }

    let is_valid = fn(line) {
        len(line) == valid_line_length
    }

    // bad: valid_line_length has not been initialized
    // is_valid("xyz")

    let valid_line_length = 3

    // good: all values initialized at this point
    part1(["abc", "de", "f"]) |> inspect()
    ```

- In compiler, "root/global" scope within a module:
    - Will have 3 maps (or one map with an enum value):
        - `imported_fields: HashMap<UStr, VariableLocation>`
        - `private_fields: HashMap<UStr, VariableLocation>`
        - `exported_fields: HashMap<UStr, VariableLocation>`

    - `let` and `var` statements will insert into `exported_fields`

    - `import Std/Enum.*` or `import Std/Enum.map` will insert into `imported_fields`

    - `import Std/Enum` or `import Std/Enum as E` will insert `Enum` or `E` into `private_fields`
        - Interpreter must interpret these statements as inserts into `GlobalEnvironment`

- **FUTURE EVOLUTION:**  Each module would have a constants

### Progress

- [x] Getting rid of ModuleId
- [x] VariableLocation changes
- [x] Hoisting
- [x] Interpreter environment changes (kept single type)
- [x] Compiler - constants & exports changes
- [ ] Rest of compiler changes re: fields
- [ ] Dropping insert_prelude_phase
- [ ] `import Something as SomethingElse`
