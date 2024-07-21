# Structs

- structs:
    ```
    module Point

    // import ...

    struct {
        x: 0,
        y: 0
    }

    let pt1 = Point{y: 2}
    let pt2 = Point{x: 1, y: 2}
    ```

- syntax: `Point{ x: 5, y: 2 }`
    - to disambiguate with `if Name {` and `for x in Name {`, no spaces allowed between struct name and fields
    - spaces allowed elsewhere as per the usual rules

- optional vs required fields:
    ```
    module Example

    struct {
        a,      // required
        b: 1,   // optional - uses default if not provided
        c,      // required
    }
    ```

- structs implementing enumerable:
    - `for i in Range{start: 0, end: 10} { println(i) }`

- get module for struct:
    ```
    import Point
    import Std/Struct

    let point = Point{x: 1, y: 2}
    assert(Point == Struct.module(point))
    ```

- dynamically create struct:
    ```
    import Point
    import Std/Struct

    let fields = {x: 1, y: 2}
    Struct.new(Point, fields)
    ```
