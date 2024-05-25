defmodule DangTest do
  use ExUnit.Case, async: true

  alias Dang

  test "v0.1 program" do
    assert 11 =
             Dang.run("""
             (let add (fn (x y)
               (let result (+ x y))
               (return result)
             ))

             (add 5 6)
             """)
  end
end
