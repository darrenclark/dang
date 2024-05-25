defmodule Dang.Eval do
  @moduledoc """
  Evaluates an AST parsed by `Dang.Parser`
  """

  def eval(ast), do: eval(ast, %{})

  defp eval([term], state) do
    {res, _} = eval_term(term, state)
    res
  catch
    {:return, val} -> val
  end

  defp eval([term | rest], state) do
    {_, state} = eval_term(term, state)
    eval(rest, state)
  catch
    {:return, val} -> val
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

  defp eval_term([:fn, arg_names | body], state) do
    {{:fn, arg_names, body}, state}
  end

  defp eval_term([:return, val], state) do
    {val, _state} = eval_term(val, state)
    throw {:return, val}
  end

  defp eval_term([func | args], state) do
    {{:fn, arg_names, body}, state} = eval_term(func, state)

    {args, state} =
      Enum.reduce(args, {[], state}, fn arg, {args, state} ->
        {arg, state} = eval_term(arg, state)
        {[arg | args], state}
      end)

    args = Enum.reverse(args)

    if length(arg_names) != length(args) do
      raise """
      Expected args: #{inspect(arg_names)}
      Got args:      #{inspect(args)}
      """
    end

    func_state = state |> Map.merge(Enum.zip(arg_names, args) |> Map.new())
    result = eval(body, func_state)
    {result, state}
  end

  defp eval_term(x, state) when is_number(x), do: {x, state}

  defp eval_term(name, state) when is_atom(name) do
    {Map.fetch!(state, name), state}
  end
end
