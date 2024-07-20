# Structs

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
