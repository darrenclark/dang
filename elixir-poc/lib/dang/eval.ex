defmodule Dang.Eval do
  @moduledoc """
  Evaluates an AST parsed by `Dang.Parser`
  """

  alias Dang.Env

  @doc """
  Evaluate an AST and return the result
  """
  def eval(ast), do: eval(ast, Env.new())

  @doc """
  Evaluate an AST and return the result

  Provided bindings become globals during evaluation
  """
  def eval(ast, bindings) when is_list(bindings) do
    eval(ast, Env.new(bindings))
  end

  def eval([term], env) do
    {res, _} = eval_term(term, env)
    res
  catch
    {:return, val} -> val
  end

  def eval([term | rest], env) do
    {_, env} = eval_term(term, env)
    eval(rest, env)
  catch
    {:return, val} -> val
  end

  def eval([], _env), do: nil

  defp eval_args(args, env) do
    {args, env} =
      Enum.reduce(args, {[], env}, fn arg, {args, env} ->
        {arg, env} = eval_term(arg, env)
        {[arg | args], env}
      end)

    args = Enum.reverse(args)
    {args, env}
  end

  defp eval_term([:+ | args], env) do
    {args, env} = eval_args(args, env)
    {Enum.sum(args), env}
  end

  defp eval_term([:- | args], env) do
    {args, env} = eval_args(args, env)
    {Enum.reduce(args, &(&2 - &1)), env}
  end

  defp eval_term([:* | args], env) do
    {args, env} = eval_args(args, env)
    {Enum.reduce(args, &(&2 * &1)), env}
  end

  defp eval_term([:/ | args], env) do
    {args, env} = eval_args(args, env)
    {Enum.reduce(args, &(&2 / &1)), env}
  end

  defp eval_term([cmp_op | args], env) when cmp_op in [:>, :>=, :<, :<=, :==, :!=] do
    {args, env} = eval_args(args, env)

    if length(args) <= 1, do: raise("expected at least 2 args for #{cmp_op}")

    {eval_cmp(cmp_op, args), env}
  end

  defp eval_term([:let, name, val], env) when is_atom(name) do
    {val, env} = eval_term(val, env)

    {val, Env.put(env, name, val)}
  end

  defp eval_term([:fn, arg_names | body], env) do
    {{:fn, arg_names, body, Env.capture(env)}, env}
  end

  defp eval_term([:return, val], env) do
    {val, _env} = eval_term(val, env)
    throw({:return, val})
  end

  defp eval_term([:if | _] = if, env), do: eval_if(if, env)

  defp eval_term([func | args], env) do
    {{:fn, arg_names, body, captured_env}, env} = eval_term(func, env)

    {args, env} = eval_args(args, env)

    if length(arg_names) != length(args) do
      raise """
      Expected args: #{inspect(arg_names)}
      Got args:      #{inspect(args)}
      """
    end

    func_env = env |> Env.function_env(captured_env, Map.new(Enum.zip(arg_names, args)))
    result = eval(body, func_env)
    {result, env}
  end

  defp eval_term(x, env) when is_number(x), do: {x, env}

  defp eval_term(name, env) when is_atom(name) do
    {Env.fetch!(env, name), env}
  end

  defp eval_cmp(_, [_]), do: true

  defp eval_cmp(:>, [lhs, rhs | rest]) do
    if lhs > rhs, do: eval_cmp(:>, [rhs | rest]), else: false
  end

  defp eval_cmp(:>=, [lhs, rhs | rest]) do
    if lhs >= rhs, do: eval_cmp(:>=, [rhs | rest]), else: false
  end

  defp eval_cmp(:<, [lhs, rhs | rest]) do
    if lhs < rhs, do: eval_cmp(:<, [rhs | rest]), else: false
  end

  defp eval_cmp(:<=, [lhs, rhs | rest]) do
    if lhs <= rhs, do: eval_cmp(:<=, [rhs | rest]), else: false
  end

  defp eval_cmp(:==, [lhs, rhs | rest]) do
    if lhs == rhs, do: eval_cmp(:==, [rhs | rest]), else: false
  end

  # != can't be evaluated by looking at each item and the next one
  # because of cases like (!= 1 3 1), hence this extra logic

  defp eval_cmp(:!=, [lhs, rhs]), do: lhs != rhs

  defp eval_cmp(:!=, args) do
    Enum.reduce_while(args, %{}, fn arg, acc ->
      if Map.has_key?(acc, arg), do: {:halt, false}, else: {:cont, Map.put(acc, arg, nil)}
    end)
    |> case do
      false -> false
      _ -> true
    end
  end

  defp eval_if([:if, cond, body | rest], env) do
    {cond, env} = eval_term(cond, env)

    if cond do
      eval_term(body, env)
    else
      eval_if(rest, env)
    end
  end

  defp eval_if([:elseif, cond, body | rest], env) do
    {cond, env} = eval_term(cond, env)

    if cond do
      eval_term(body, env)
    else
      eval_if(rest, env)
    end
  end

  defp eval_if([:else, body], env) do
    eval_term(body, env)
  end

  defp eval_if([], env), do: {nil, env}
end
