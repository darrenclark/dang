use common::{assert_raises, assert_runs};

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

        assert(trim("   hello world ") == "hello world")

        assert(str_replace("hello world", "world", "bob") == "hello bob")

        assert(str_find("hello world", "world") == 6)
        assert(str_find("hello world", "bob") == nil)

        assert(str_rfind("hahaha", "ha") == 4)
        assert(str_rfind("hahaha", "404") == nil)

        assert(str_starts_with?("hello world", "hello"))
        assert(!str_starts_with?("hello world", "hi"))
        assert(str_ends_with?("hello world", "world"))
        assert(!str_ends_with?("hello world", "earth"))
        "#
    };
}

#[test]
fn test_symbol_literals() {
    assert_runs! {
        r#"
        assert(:hello == symbol("hello"))
        assert(:hello_world == symbol("hello_world"))
        assert(:hello_world? == symbol("hello_world?"))
        assert(:"hello world #@$" == symbol("hello world #@$"))
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
fn test_append() {
    assert_runs! {
        r#"
        var list = [1, 2, 3]
        list = append(list, 4)
        assert(list == [1, 2, 3, 4])
        "#
    };
}

#[test]
fn test_reverse() {
    assert_runs! {
        r#"
        assert(reverse([1, 2, 3]) == [3, 2, 1])
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
fn type_introspection() {
    assert_runs! {
        r#"
        assert(type(5) == "int")
        assert(type("hello") == "string")
        assert(type([1, 2, 3]) == "list")
        assert(type({a: 1, b: 2}) == "dict")
        assert(type(fn() {}) == "function")
        assert(type(nil) == "nil")

        assert(is_int(5))
        assert(is_string("hello"))
        assert(is_list([1, 2, 3]))
        assert(is_dict({a: 1, b: 2}))

        assert(is_tuple((1, 2)))
        assert(is_tuple((1, 2), 2))
        assert(!is_tuple((1, 2, 3), 2))
        assert(!is_tuple("12", 2))

        assert(!is_function(5))
        assert(!is_nil(5))
        assert(!is_list("hello"))
        "#
    };
}

#[test]
fn test_scoping() {
    assert_runs! {
        r#"
        for x in [1, 2, 3] { print(x) }
        for x in ["a", "b", "c"] { print(x) }

        var y = 1
        if true {
          var y = 5
          y = y + 1
          assert(y == 6)
        }
        assert(y == 1)

        let curry_add = fn(a) {
            fn(b) {
                a + b
            }
        }
        let add3 = curry_add(3)
        let add5 = curry_add(5)
        assert(add3(1) == 4)
        assert(add5(8) == 13)
        "#
    };
}

#[test]
fn test_scoping_edge_case() {
    assert_runs! {
        r#"
        var a = "hello"
        let edge_case = fn () {
            let r = fn () {
                let prev = a
                a = "world"
                prev
            }
            var a = 123
            r
        }

        assert(edge_case()() == "hello")
        assert(a == "world")
        "#
    };
}

#[test]
fn test_upvalues_more_than_two_functions_deep() {
    assert_runs! {
        r#"
        let outer = fn(x) {
            let middle = fn(y) {
                let inner = fn(z) {
                    x + y + z
                }
                inner
            }
            middle
        }

        assert(outer(1)(2)(3) == 6)
        "#
    };
}

#[test]
fn test_capturing_loop_variables() {
    assert_runs! {
        r#"
        var funcs = []
        for i in [1, 2, 3] {
            funcs = funcs + [fn() { i }]
        }
        let results = []
        for f in funcs {
            results = results + [f()]
        }
        assert(results == [1, 2, 3])
        "#
    };

    assert_runs! {
        r#"
        var funcs = []
        for (_, i) in [(nil, 1), (nil, 2), (nil, 3)] {
            funcs = funcs + [fn() { i }]
        }
        let results = []
        for f in funcs {
            results = results + [f()]
        }
        assert(results == [1, 2, 3])
        "#
    };
}

#[test]
fn test_variable_hoisting_at_root() {
    assert_raises! {
        (3, "Global not found: (u!(\"(run)\"), u!(\"i\"))"),
        r#"
        let incr = fn() {
            i = i + 1
            i
        }

        // should fail - `incr()` access `i`, but `i` hasn't been initialized yet
        assert(incr() == 1)

        var i = 0
        "#
    }

    assert_runs! {
        r#"
        let incr = fn() {
            i = i + 1
            i
        }

        var i = 0

        assert(incr() == 1)
        assert(incr() == 2)
        "#
    }
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

#[test]
fn test_tuples() {
    assert_runs! {
        r#"
        let t0 = ()
        let t1 = (42,)
        let t2 = (19,99)
        let t3 = (1,2,3,)

        assert(len(t0) == 0)
        assert(len(t1) == 1)
        assert(len(t2) == 2)
        assert(len(t3) == 3)

        assert(t1[0] == 42)
        assert(get(t2, 1) == 99)
        "#
    };
}

#[test]
fn test_tuple_unpacking() {
    assert_runs! {
        r#"
        let (a, b, c) = (5, "two", 3)

        assert(a == 5)
        assert(b == "two")
        assert(c == c)

        let (x, (y, z)) = (1, (2, 3))
        assert(x == 1)
        assert(y == 2)
        assert(z == 3)

        let pairs = [("red", 5), ("green", 2), ("blue", 114)]
        var copied = []
        for (k, v) in pairs {
            copied = copied + [(k, v)]
        }
        assert(copied == pairs)
        "#
    };

    assert_raises! {
        (2, "Pattern match failed, got (1, 2)"),
        r#"
        let (a, b, c) = (1, 2)
        "#
    }

    assert_raises! {
        (2, "Pattern match failed, got [1, 2, 3]"),
        r#"
        let (a, b, c) = [1, 2, 3]
        "#
    }
}

#[test]
fn test_wildcards_in_patterns() {
    assert_runs! {
        r#"
        let (_, _, c) = (1, 2, 3)
        assert(c == 3)
        "#
    };
}

#[test]
fn test_dicts() {
    assert_runs! {
        r#"
        let k = "c"

        let d = {
            a: 1,
            " b " => 2,
            k => 3,
            [1,2] => 4
        }

        assert(d[:a] == 1)
        assert(d.a == 1)
        assert(d[" b "] == 2)
        assert(d["c"] == 3)
        assert(d[[1, 2]] == 4)
        "#
    };
}

#[test]
fn test_iterating_dict() {
    assert_runs! {
        r#"
        let d = {
            "a" => 1,
            "b" => 2,
            "c" => 3
        }

        var keys = ""
        var sum = 0

        for (k, v) in d {
            keys = keys + k
            sum = sum + v
        }

        assert(keys == "abc")
        assert(sum == 6)
        "#
    };
}

#[test]
fn test_trailing_commas_in_literals() {
    assert_runs! {
        r#"
        let a1 = [1, 2, 3,]
        let a2 = [
            1,
            2,
            3,
        ]
        assert(a1 == [1, 2, 3])
        assert(a2 == [1, 2, 3])

        let b1 = {a: 1, b: 2,}
        let b2 = {
            a: 1,
            b: 2,
        }
        assert(b1 == {a: 1, b: 2})
        assert(b2 == {a: 1, b: 2})
        "#
    };
}

#[test]
fn test_module_field_access() {
    assert_runs! {
        r#"
        import Std/Enum

        assert(map == Enum.map)
        "#
    };
}

#[test]
fn test_modules_dont_overwrite_other_modules_globals() {
    assert_runs! {
        r#"
        import Tests/NameConflictOne.conflict
        import Tests/NameConflictTwo

        // the `conflict` from NameConflictOne should not have been
        // overwritten by the `conflict` from NameConflictTwo
        assert(conflict == "one")
        "#
    };
}

#[test]
fn test_can_refer_to_module_with_short_name_in_same_file() {
    assert_runs! {
        "TestCase/ShortName",
        r#"
        module TestCase/ShortName

        assert(ShortName == symbol("TestCase/ShortName"))
        "#
    };
}

#[test]
fn test_module_imports_can_be_referred_to() {
    assert_runs! {
        r#"
        import Tests/NameConflictOne
        import Tests/NameConflictTwo

        assert(NameConflictOne == symbol("Tests/NameConflictOne"))
        assert(NameConflictTwo == symbol("Tests/NameConflictTwo"))
        "#
    };
}

#[test]
fn struct_from_current_file() {
    assert_runs! {
        r#"
        module TestCase/Point

        struct { x, y }

        Point{x: 0, y: 1}
        "#
    };
}

#[test]
fn struct_from_imported_file() {
    assert_runs! {
        r#"
        import Tests/Note

        Note{author: "Darren", text: "It works!"}
        "#
    };
}

#[ignore = "requires compiler to output better errors"]
#[test]
fn struct_raises_if_a_missing_field_is_not_provided() {
    assert_raises! {
        (4, "required struct field `author` not provided"),
        r#"
        import Tests/Note

        Note{text: "It works!"}
        "#
    };
}

#[ignore = "requires compiler to output better errors"]
#[test]
fn struct_raises_if_an_unknown_field_is_provided() {
    assert_raises! {
        (4, "unknown field `four_oh_four` provided for struct `Tests/Note`"),
        r#"
        import Tests/Note

        Note{
          text: "It works!",
          author: "Darren",
          four_oh_four: "404"
        }
        "#
    };
}

#[test]
fn struct_field_access() {
    assert_runs! {
        r#"
        import Tests/Note

        let n = Note{
          text: "It works!",
          author: "Darren",
        }

        assert(n.text == "It works!")
        assert(n.author == "Darren")
        "#
    };

    assert_raises! {
        (9, "field `foo` not found in struct `Tests/Note`"),
        r#"
        import Tests/Note

        let n = Note{
          text: "It works!",
          author: "Darren",
        }

        n.foo
        "#
    };
}

#[test]
fn struct_defaults() {
    assert_runs! {
        r#"
        module TestCase/Defaults

        struct {
            a: "one",
            b: "two",
        }

        assert(Defaults{}.a == "one")
        assert(Defaults{a: "A"}.a == "A")
        assert(Defaults{b: "Z"}.a == "one")
        "#
    }
}

#[test]
fn structs_implementing_iter() {
    assert_runs! {
        r#"
        module TestCase/Repeat

        struct {
            element,
            times,
        }

        let __iter__ = fn (repeat) {
            var i = 0

            fn () {
                if i < repeat.times {
                    i = i + 1
                    (some, repeat.element)
                }
            }
        }

        var result = ""
        for e in Repeat{element: "a", times: 3} {
            result = result + e
        }

        assert(result == "aaa")

        result = ""
        let it = iter(Repeat{element: "b", times: 2})
        assert(it() == (some, "b"))
        assert(it() == (some, "b"))
        assert(it() == nil)
        "#
    }
}

#[test]
fn assigning_to_fields() {
    assert_runs! {
        r#"
        module TestCase/Fields

        struct {
            one,
            two,
        }

        var f = Fields{one: {a: "a", b: [1, 2, 3]}, two: 2}
        f.one[:b][1] = 36

        let expected = Fields{one: {a: "a", b: [1, 36, 3]}, two: 2}
        assert(f == expected)
        "#
    }
}

#[test]
fn empty_functions_return_nil() {
    assert_runs! {
        r#"
        let f = fn() {}
        assert(f() == nil)
        "#
    }
}

#[test]
fn exception_raised_when_function_given_wrong_number_of_args() {
    assert_raises! {
        (3, "expected 2 arguments, got 1"),
        r#"
        let f = fn(x, y) { x + y }
        f(1)
        "#
    }
}

#[test]
fn shadowing_imports_works_correctly_even_in_same_let_or_var() {
    assert_runs! {
        r#"
        import Std/Enum.count

        let count_list = fn(l) {
          let count = l |> count()
          count
        }
        count_list([1, 2, 3])
        "#
    }
}

#[test]
fn math() {
    assert_runs! {
        r#"
        assert(Math.lcm(4, 6) == 12)
        assert(Math.gcd(4, 6) == 2)
        assert(Math.rem(8, 3) == 2)
        "#
    };

    assert_raises! {
        (3, "rem: division by zero"),
        r#"
        Math.rem(5, 0)
        "#
    };
}

#[test]
fn matches() {
    assert_runs! {
        r#"
        let m = fn (x) {
            match x {
                "one" -> 1
                (:abs, r) if r > 0 -> r
                (:abs, r) if r < 0 -> -r
                (:abs, 0) -> 0
                (_, r) -> r,
                (:complex, a, (b, c)) -> {
                  var s = 0
                  for x in range(b, c) {
                    s = s + a
                  }
                  s
                }
                r -> r
            }
        }

        assert(m((1, 2)) == 2)
        assert(m(4) == 4)
        assert(m("one") == 1)
        assert(m((:abs, 5)) == 5)
        assert(m((:abs, -5)) == 5)
        assert(m((:abs, 0)) == 0)
        assert(m((:complex, 2, (1, 3))) == 4)
        "#
    }
}

#[test]
fn zip() {
    assert_runs! {
        r#"
        let a = [1, 2, 3]
        let b = ["a", "b", "c", "d"]

        assert(zip(a, b) == [(1, "a"), (2, "b"), (3, "c")])
        "#
    }
}
