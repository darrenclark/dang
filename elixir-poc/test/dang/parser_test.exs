defmodule Dang.ParserTest do
  use ExUnit.Case, async: true

  alias Dang.Parser

  test "parser works" do
    assert [
             [:one, :two2, :three]
           ] = Parser.parse("( one two2 three )")
  end

  test "works recursively" do
    assert [
             [:one, [:two2, :three]]
           ] = Parser.parse("( one (two2 three) )")
  end

  test "works multiple sexpr in a row" do
    assert [
             [:a, :b],
             [:c, :d]
           ] = Parser.parse("(a b) (c d)")
  end

  test "can parse v0.1" do
    assert [
             [
               :let,
               :add,
               [:fn, [:x, :y], [:let, :result, [:+, :x, :y]], [:return, :result]]
             ],
             [:add, 5, 6]
           ] =
             Parser.parse("""
             (let add (fn (x y)
               (let result (+ x y))
               (return result)
             ))

             (add 5 6)
             """)
  end

  test "can parse a string" do
    assert [
             [:print, "Hello, world"]
           ] =
             Parser.parse("""
             (print "Hello, world")
             """)
  end

  test "parses escape sequences correctly" do
    assert [
      [:print, "alice\nbob"]
    ] = Parser.parse(~s|(print "alice\\nbob")|)
  end
end
