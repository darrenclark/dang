defmodule Dang.EvalTest do
  use ExUnit.Case, async: true

  alias Dang.Eval

  test "addition" do
    assert 5 = Eval.eval([[:+, 3, 2]])
  end
end
