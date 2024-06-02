defmodule Dang.BuiltinsTest do
  use ExUnit.Case, async: true

  # TODO: Cleanup this file to only use run
  alias Dang.Eval
  import Dang, only: [run: 1]

  test "int" do
    assert 5 = run(~s|(int "5")|)
    assert 5 = run(~s|(int 5)|)
  end

  test "trim" do
    assert "hello" = run(~s|(trim "\thello\n")|)
  end

  test "split" do
    assert ["hello", "world"] = run(~s|(split "hello world" " ")|)
  end

  test "readfile" do
    File.write!("/tmp/dang-readfile-test.txt", "alice\nbob\ncharlie\n")

    assert "alice\nbob\ncharlie\n" = run(~s|(readfile "/tmp/dang-readfile-test.txt")|)
  end

  test "len" do
    assert 2 = Eval.eval([[:len, [:list, 3, 2]]])
    assert 0 = Eval.eval([[:len, [:list]]])

    assert 3 = Eval.eval([[:len, "abc"]])
    assert 0 = Eval.eval([[:len, ""]])
  end

  test "at" do
    assert 3 = Eval.eval([[:at, 0, [:list, 3, 2]]])
    assert 2 = Eval.eval([[:at, 1, [:list, 3, 2]]])
    assert %RuntimeError{} = catch_error(Eval.eval([[:at, -1, [:list, 3, 2]]]))
    assert %RuntimeError{} = catch_error(Eval.eval([[:at, 2, [:list, 3, 2]]]))

    assert "a" = Eval.eval([[:at, 0, "ab"]])
    assert "b" = Eval.eval([[:at, 1, "ab"]])
    assert %RuntimeError{} = catch_error(Eval.eval([[:at, -1, "ab"]]))
    assert %RuntimeError{} = catch_error(Eval.eval([[:at, 2, "ab"]]))
  end

  test "first" do
    assert 3 = Eval.eval([[:first, [:list, 3, 2]]])
    assert nil == Eval.eval([[:first, [:list]]])

    assert "a" = Eval.eval([[:first, "abc"]])
    assert nil == Eval.eval([[:first, ""]])
  end

  test "last" do
    assert 2 = Eval.eval([[:last, [:list, 3, 2]]])
    assert nil == Eval.eval([[:last, [:list]]])

    assert "c" = Eval.eval([[:last, "abc"]])
    assert nil == Eval.eval([[:last, ""]])
  end
end
