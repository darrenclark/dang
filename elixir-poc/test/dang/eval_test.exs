defmodule Dang.EvalTest do
  use ExUnit.Case, async: true

  alias Dang.Eval

  test "addition" do
    assert 5 = Eval.eval([[:+, 3, 2]])

    assert "Hello, world!" = Eval.eval([[:+, "Hello,", " ", "world!"]])

    assert [1, 2, 3, 4] = Eval.eval([[:+, [:list, 1, 2], [:list, 3, 4]]])
  end

  test "variables" do
    code = [
      [:let, :x, 3],
      [:let, :y, 2],
      [:+, :x, :y]
    ]

    assert 5 = Eval.eval(code)
  end

  test "functions" do
    code = [
      [
        :let,
        :add,
        [:fn, [:x, :y], [:let, :result, [:+, :x, :y]], [:return, :result]]
      ],
      [:add, 5, 6]
    ]

    assert 11 = Eval.eval(code)
  end

  test "ifs" do
    code = [
      [:if, [:>, :x, 0], 1, :elseif, [:==, :x, 0], 0, :else, -1]
    ]

    assert 1 == Eval.eval(code, x: 5)
    assert 1 == Eval.eval(code, x: 3)
    assert 0 == Eval.eval(code, x: 0)
    assert -1 == Eval.eval(code, x: -32)

    code = [
      [:if, [:>=, :x, 0], 42, :else, 21]
    ]

    assert 42 == Eval.eval(code, x: 90)
    assert 42 == Eval.eval(code, x: 0)
    assert 21 == Eval.eval(code, x: -23)

    code = [
      [:if, [:<=, :x, 0], 11]
    ]

    assert nil == Eval.eval(code, x: 90)
    assert 11 == Eval.eval(code, x: 0)
    assert 11 == Eval.eval(code, x: -23)
  end

  test "!= evaluation" do
    assert true == Eval.eval([[:!=, 1, 3]])
    assert false == Eval.eval([[:!=, 1, 1]])
    assert false == Eval.eval([[:!=, 1, 3, 1]])
    assert true == Eval.eval([[:!=, 1, 3, 5]])
  end

  test "special atoms evaluate to themselves" do
    # (let returns it's value)
    assert nil == Eval.eval([[:let, :x, nil]])
    assert true == Eval.eval([[:let, :x, true]])
    assert false == Eval.eval([[:let, :x, false]])
  end

  test "truthiness or falsiness" do
    code = [
      [:if, :input, 1, :else, 0]
    ]

    assert 1 == Eval.eval(code, input: true)
    assert 0 == Eval.eval(code, input: false)
    assert 0 == Eval.eval(code, input: nil)
  end

  test "|| returns the first truthy value, or nil" do
    code = [
      [:||, :a, :b, :c]
    ]

    assert true == Eval.eval(code, a: true, b: nil, c: false)
    assert 1 == Eval.eval(code, a: false, b: nil, c: 1)
    assert nil == Eval.eval(code, a: false, b: false, c: false)
    assert "a" == Eval.eval(code, a: "a", b: "b", c: "c")
  end

  test "&& returns last truthy value, or false" do
    code = [
      [:&&, :a, :b, :c]
    ]

    assert "hello" == Eval.eval(code, a: true, b: "a", c: "hello")
    assert false == Eval.eval(code, a: true, b: nil, c: "hello")
  end

  test "list returns a list" do
    code = [
      [:list, :a, "b", 3]
    ]

    assert ["a", "b", 3] == Eval.eval(code, a: "a")
  end

  test "do evaluates its body sequentially" do
    code = [
      [:do, [:let, :x, 5], [:let, :y, 9], [:*, :x, :y]]
    ]

    assert 45 = Eval.eval(code)
  end
end
