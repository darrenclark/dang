use common::assert_runs;

mod common;

#[test]
fn test_math_bedmas() {
    assert_runs! {
        "
        let x = 1 + 2 * 3 * (5 - 1) / 2
        assert(x == 13)
        "
    };
}

#[test]
fn test_function_calls() {
    assert_runs! {
        "
        let f = fn(x, y) {
            (x + 1) * y
        }

        assert(f(1, 2) == 4)
        "
    };
}

#[test]
fn test_string_functions() {
    assert_runs! {
        r#"
        assert(split("a,,c", ",") == ["a", "", "c"])

        assert(str_replace("hello world", "world", "bob") == "hello bob")

        assert(str_find("hello world", "world") == 6)
        assert(str_find("hello world", "bob") == nil)

        assert(str_rfind("hahaha", "ha") == 4)
        assert(str_rfind("hahaha", "404") == nil)
        "#
    };
}

#[test]
fn test_for_iteration() {
    assert_runs! {
        r#"
        // lists
        var sum = 0
        for x in [1, 2, 3, 4] {
            sum = sum + x
        }
        assert(sum == 10)

        // strings
        var chars = []
        for y in "abcdef" {
            chars = chars + [y]
        }
        assert(chars == ["a", "b", "c", "d", "e", "f"])
        "#
    };
}

#[test]
fn test_len() {
    assert_runs! {
        r#"
        assert(len("abcdef") == 6)
        assert(len([1, 2, 3]) == 3)
        "#
    };
}

#[test]
fn test_get() {
    assert_runs! {
        r#"
        assert(get("abcdef", 3) == "d")
        assert(get([1, 2, 3], 1) == 2)

        //TODO: assert_raise or something
        //assert(get("", 2) == nil)

        let grid = [
            "abc",
            "def",
            "ghi"
        ]
        assert(get(grid, 1, 1) == "e")
        "#
    };
}

#[test]
fn test_casting() {
    assert_runs! {
        r#"
        assert(int(5) == 5)
        assert(int("11") == 11)
        assert(int(true) == 1)
        assert(int(false) == 0)

        //TODO: assert_raise or something
        //assert(int("abc") == nil)
        "#
    };
}

#[test]
#[ignore = "test fails, x from first loop isn't removed from scope"]
fn test_scoping() {
    assert_runs! {
        r#"
        for x in [1, 2, 3] { print(x) }
        for x in ["a", "b", "c"] { print(x) }
        "#
    };
}
