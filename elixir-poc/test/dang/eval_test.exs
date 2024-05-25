defmodule Dang.EvalTest do
  use ExUnit.Case, async: true

  alias Dang.Eval

  test "addition" do
    assert 5 = Eval.eval([[:+, 3, 2]])
  end

  test "variables" do
    code = [
      [:let, :x, 3],
      [:let, :y, 2],
      [:+, :x, :y]
    ]
    assert 5 = Eval.eval(code)
  end
end
