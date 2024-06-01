defmodule Dang do
  @moduledoc """
  Main entrypoint
  """

  def main([]) do
    IO.puts("ERROR: Pass program in arguments, i.e.:")
    IO.puts("./dang '(print \"Hello, world\")'")
  end

  def main(argv) do
    run(Enum.join(argv, "\n"))
  end

  def run(input) do
    input
    |> Dang.Parser.parse()
    |> Dang.Eval.eval()
  end
end
