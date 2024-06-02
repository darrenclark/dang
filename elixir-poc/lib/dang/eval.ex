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

  defp eval_term([op | args], env) when op in [:and, :&&] do
    {args, env} = eval_args(args, env)
    {eval_and(args), env}
  end

  defp eval_term([op | args], env) when op in [:or, :||] do
    {args, env} = eval_args(args, env)
    {eval_or(args), env}
  end

  defp eval_term([:let, name, val], env) when is_atom(name) do
    {val, env} = eval_term(val, env)

    {val, Env.put(env, name, val)}
  end

  defp eval_term([:var, name, val], env) when is_atom(name) do
    {val, env} = eval_term(val, env)

    {val, Env.put(env, name, val)}
  end

  defp eval_term([:set, name, val], env) when is_atom(name) do
    {val, env} = eval_term(val, env)

    {val, Env.put(env, name, val)}
  end

  defp eval_term([:get, val], env) do
    {val, env} = eval_term(val, env)
    {val, env}
  end

  defp eval_term([:fn, arg_names | body], env) do
    {{:fn, arg_names, body, Env.capture(env)}, env}
  end

  defp eval_term([:for, elem_name, enum, body], env) do
    {enum, env} = eval_term(enum, env)

    len = Dang.Enum.len(enum)

    if len > 0 do
      env =
        Enum.reduce(0..(len - 1), env, fn i, env ->
          env = Dang.Env.put(env, elem_name, Dang.Enum.at(enum, i))
          {_, env} = eval_term(body, env)
          env
        end)

      {nil, env}
    else
      {nil, env}
    end
  end

  defp eval_term([:do | body], env) do
    {result, env} = eval_args(body, env)
    {List.last(result), env}
  end

  defp eval_term([:return, val], env) do
    {val, _env} = eval_term(val, env)
    throw({:return, val})
  end

  defp eval_term([:if | _] = if, env), do: eval_if(if, env)

  defp eval_term([func | args], env) do
    case eval_term(func, env) do
      {{:fn, arg_names, body, captured_env}, env} ->
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

      {{:builtin, mod, fun, extra_args}, env} ->
        {args, env} = eval_args(args, env)
        result = apply(mod, fun, [args | extra_args])
        {result, env}
    end
  end

  defp eval_term(x, env) when is_number(x), do: {x, env}

  defp eval_term(atom, env) when atom in [nil, true, false] do
    {atom, env}
  end

  defp eval_term(name, env) when is_atom(name) do
    {Env.fetch!(env, name), env}
  end

  defp eval_term(bin, env) when is_binary(bin), do: {bin, env}

  defp eval_and([arg]), do: arg || false
  defp eval_and([arg | rest]) when arg not in [nil, false], do: eval_and(rest)
  defp eval_and([_arg | _rest]), do: false
  defp eval_and([]), do: false

  defp eval_or([arg | _rest]) when arg not in [nil, false], do: arg
  defp eval_or([_arg | rest]), do: eval_or(rest)
  defp eval_or([]), do: nil

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
