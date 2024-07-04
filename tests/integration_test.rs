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
        var s = 0
        for x in [1, 2, 3, 4] {
            s = s + x
        }
        assert(s == 10)

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

#[test]
fn test_regex() {
    assert_runs! {
        r#"
        assert(regex_match("hello", "hel+o"))

        // TODO: support capture groups

        assert(regex_find("hello", "l{2}") == "ll")
        assert(regex_find("hello", "he(l+)o") == "hello")
        assert(regex_find("hello", "h{2}") == nil)

        assert(regex_find_index("hello", "l{2}") == [2, 4])
        assert(regex_find_index("hello", "he(l+)o") == [0, 5])
        assert(regex_find_index("hello", "zzz") == nil)
        "#
    };
}

#[test]
fn test_logical_and_or() {
    assert_runs! {
        r#"
        assert((5 || 2) == 5)
        assert((nil || 2) == 2)
        assert((nil || false) == false)

        assert((5 && 2) == 2)
        assert((nil && 2) == nil)
        assert((false && nil) == false)

        // short circuiting
        5 || raise("this raise should not be evaluated")
        "#
    };
}

#[test]
fn test_newline_scenarios() {
    assert_runs! {
        r#"
        let x =
            5 +
            3; let y = 9

        assert(
            x +
            y ==
            17
        )

        // ensure the array [5] isn't interpreted as a subscript
        let a = [1, 2, 3]
        [5]
        assert(a == [1, 2, 3])
        "#
    };
}

#[test]
fn test_subscripting() {
    assert_runs! {
        r#"
        let x = [
            "abc",
            "def",
            "ghi"
        ]

        assert(x[2] == "ghi")
        assert(x[0][1] == "b")
        assert(x[1][0] == "d")
        assert(x[1][2] == "f")
        "#
    };
}

#[test]
fn test_piping() {
    assert_runs! {
        r#"
        let numbers = [
            "one",
            "two",
            "three",
            "four"
        ]

        let r =
            numbers
            |> filter(fn(x) { len(x) == 3 })
            |> map(str_reverse)

        assert(r == ["eno", "owt"])
        "#
    };
}
