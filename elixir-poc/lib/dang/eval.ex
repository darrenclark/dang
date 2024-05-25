defmodule Dang.Eval do
  @moduledoc """
  Evaluates an AST parsed by `Dang.Parser`
  """

  alias Dang.Env

  def eval(ast), do: eval(ast, Env.new())

  defp eval([term], env) do
    {res, _} = eval_term(term, env)
    res
  catch
    {:return, val} -> val
  end

  defp eval([term | rest], env) do
    {_, env} = eval_term(term, env)
    eval(rest, env)
  catch
    {:return, val} -> val
  end

  defp eval([], _env), do: nil

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

  defp eval_term([:let, name, val], env) when is_atom(name) do
    {val, env} = eval_term(val, env)

    {val, Env.put(env, name, val)}
  end

  defp eval_term([:fn, arg_names | body], env) do
    {{:fn, arg_names, body, Env.capture(env)}, env}
  end

  defp eval_term([:return, val], env) do
    {val, _env} = eval_term(val, env)
    throw {:return, val}
  end

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
end
