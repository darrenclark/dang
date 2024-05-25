defmodule Dang.ParserTest do
  use ExUnit.Case, async: true

  alias Dang.Parser

  test "parser works" do
    assert [] = Parser.parse(1)
  end
end
