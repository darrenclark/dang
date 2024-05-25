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

  test "globals can be defined in any order" do
    assert 22 =
             Dang.run("""
             (let domath (fn (x y)
               (let result (+ (double x) (double y)))
               (return result)
             ))

             (let double (fn (x) (return (+ x x))))

             (domath 5 6)
             """)
  end

  test "local variables don't leak across function calls" do
    assert catch_error(
             Dang.run("""
             (let test (fn (x)
               (let result (tryx x))
               (return result)
             ))

             (let tryx (fn (notx) (return (+ x x))))

             (test 5)
             """)
           ) == %RuntimeError{message: "Variable :x not found"}
  end
end
