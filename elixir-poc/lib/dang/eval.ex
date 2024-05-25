defmodule Dang.Eval do
  @moduledoc """
  Evaluates an AST parsed by `Dang.Parser`
  """

  def eval(ast), do: eval(ast, %{})

  defp eval([term], state) do
    {res, _} = eval_term(term, state)
    res
  end

  defp eval([term | rest], state) do
    {_, state} = eval_term(term, state)
    eval(rest, state)
  end

  defp eval([], _state), do: nil

  defp eval_term([:+, a, b], state) do
    {a, state} = eval_term(a, state)
    {b, state} = eval_term(b, state)

    {a + b, state}
  end

  defp eval_term([:let, name, val], state) when is_atom(name) do
    {val, state} = eval_term(val, state)

    {val, Map.put(state, name, val)}
  end

  defp eval_term(x, state) when is_number(x), do: {x, state}

  defp eval_term(name, state) when is_atom(name) do
    {Map.fetch!(state, name), state}
  end
end
