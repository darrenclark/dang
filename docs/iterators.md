# Iterators

## Custom iterator ideas

- via function (nil terminated)
    ```
    module Std/Range {start:0, stop: 0}

    let iter = fn (range) {
        var current = range.start

        fn () {
            if current >= range.stop {
                nil
            } else {
                current = current + 1
                current
            }
        }
    }
    ```

- via function (tuples + symbols)
    ```
    module Std/Range {start:0, stop: 0}

    let iter = fn (range) {
        var current = range.start

        fn () {
            if current >= range.stop {
                :halt
            } else {
                current = current + 1
                (:cont, current)
            }
        }
    }
    ```

- via function (maps)
    ```
    module Std/Range {start:0, stop: 0}

    let iter = fn (range) {
        var current = range.start

        fn () {
            if current >= range.stop {
                nil
            } else {
                current = current + 1
                {elem: current}
            }
        }
    }
    ```

- via function - using Iter.done()/.next() functions
    ```
    module Std/Range {start:0, stop: 0}

    import Std/Iter

    let iter = fn (range) {
        var current = range.start

        fn () {
            if current >= range.stop {
                Iter.done()
            } else {
                current = current + 1
                Iter.next(current)
            }
        }
    }
    ```

- via function - using Iter.yield function
    ```
    module Std/Range {start:0, stop: 0}

    import Std/Iter

    let iter = fn (range) {
        var current = range.start

        fn () {
            if current < range.end {
                current = current + 1
                Iter.yield(current)
            }
        }
    }
    ```

- via struct
    ```
    module Std/Range {start:0, stop: 0}

    import Std/Iter

    let iter = fn (range) {
        var current = range.start

        Iter{
            has_next: fn () {
                current < range.stop
            },
            next: fn () {
                let r = current
                current = current + 1
                r
            }
        }
    }
    ```
