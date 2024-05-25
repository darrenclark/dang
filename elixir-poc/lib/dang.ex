defmodule Dang do
  @moduledoc """
  Main entrypoint
  """

  def main(_argv) do
    IO.puts("Hello")
  end

  def run(input) do
    input
    |> Dang.Parser.parse()
    |> Dang.Eval.eval()
  end
end
