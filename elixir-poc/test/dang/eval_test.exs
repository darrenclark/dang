defmodule Dang.EvalTest do
  use ExUnit.Case, async: true

  alias Dang.Eval

  test "addition" do
    assert 5 = Eval.eval([[:+, 3, 2]])

    assert "Hello, world!" = Eval.eval([[:+, "Hello,", " ", "world!"]])
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
end
