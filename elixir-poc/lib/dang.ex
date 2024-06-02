defmodule Dang do
  @moduledoc """
  Main entrypoint
  """

  def main([]) do
    IO.puts("ERROR: Pass path to program in arguments, i.e.:")
    IO.puts("")
    IO.puts("  ./dang program.dang")
    IO.puts("")
    IO.puts("Use - to read from stdin:")
    IO.puts("")
    IO.puts("  echo '(print \"Hello, world\")' | ./dang -")
    IO.puts("")
    System.halt(1)
  end

  def main([path | argv]) do
    run(File.read!(path), argv: argv)
  end

  def run(input, bindings \\ []) do
    input
    |> Dang.Parser.parse()
    |> Dang.Eval.eval(bindings)
  end
end
