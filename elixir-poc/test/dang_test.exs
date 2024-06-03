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

  test "can do math" do
    assert 5 == Dang.run("(- (/ (* (+ 1 3 2) 8) 4) 5 2)")
  end

  test "if statements work" do
    assert 9 ==
             Dang.run("""
             (let x 5)
             (let y 10)
             (if (and (== x 5) (> y 7)) 9 else 3)
             """)
  end

  test "can implement map" do
    assert [1, 4, 9] ==
             Dang.run("""
             (let map (fn (enum func)
               (let result (list))
               (for x enum (do
                 (set result (+ result (list (func x))))
               ))
               (return result)
             ))

             (map (list 1 2 3) (fn (x) (* x x)))
             """)
  end

  test "can implement filter" do
    assert [2, 3] ==
             Dang.run("""
             (let filter (fn (enum func)
               (let result (list))
               (for x enum (do
                 (if (func x)
                   (set result (+ result (list x))))
               ))
               (return result)
             ))

             (filter (list 1 2 3) (fn (x) (>= x 2)))
             """)
  end

  test "is? function" do
    assert true == Dang.run("(is? (list 1 2 3) enum)")
    assert false == Dang.run("(is? false enum)")
    assert true == Dang.run("(is? (/ 3 2) number)")
    assert false == Dang.run("(is? (/ 3 2) int)")
    assert true == Dang.run("(is? (/ 3 2) float)")
    assert false == Dang.run("(is? (/ 3 2) str)")
    assert true == Dang.run("(is? \"hello\" str)")
    assert true == Dang.run("(is? \"hello\" enum)")
  end

  test "implicit function returns" do
    assert 11 =
             Dang.run("""
             (let add (fn (x y) (+ x y)))

             (add 5 6)
             """)
  end
end
