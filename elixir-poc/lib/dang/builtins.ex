defmodule Dang.Builtins do
  def builtins do
    %{
      :+ => {:builtin, __MODULE__, :add, []},
      :- => {:builtin, __MODULE__, :sub, []},
      :* => {:builtin, __MODULE__, :mul, []},
      :/ => {:builtin, __MODULE__, :div, []},
      :list => {:builtin, __MODULE__, :list, []},
      :>= => {:builtin, __MODULE__, :cmp, [:>=]},
      :> => {:builtin, __MODULE__, :cmp, [:>]},
      :<= => {:builtin, __MODULE__, :cmp, [:<=]},
      :< => {:builtin, __MODULE__, :cmp, [:<]},
      :== => {:builtin, __MODULE__, :cmp, [:==]},
      :!= => {:builtin, __MODULE__, :cmp_ne, []},
      :str => {:builtin, __MODULE__, :str, []},
      :print => {:builtin, __MODULE__, :print, []},
    }
  end

  def add(args) do
    Enum.reduce(args, &add/2)
  end

  defp add(b, a) when is_binary(a) and is_binary(b), do: a <> b
  defp add(b, a) when is_list(a) and is_list(b), do: a ++ b
  defp add(b, a), do: a + b

  def sub(args), do: Enum.reduce(args, &(&2 - &1))

  def mul(args), do: Enum.reduce(args, &(&2 * &1))

  def div(args), do: Enum.reduce(args, &(&2 / &1))

  def list(args), do: args

  def cmp([_], _op), do: true

  def cmp([lhs, rhs | rest], op) do
    if apply(Kernel, op, [lhs, rhs]), do: cmp([rhs | rest], op), else: false
  end

  def cmp_ne([lhs, rhs]), do: lhs != rhs

  def cmp_ne(args) do
    # != can't be evaluated by looking at each item and the next one
    # because of cases like (!= 1 3 1), hence this extra logic
    Enum.reduce_while(args, %{}, fn arg, acc ->
      if Map.has_key?(acc, arg), do: {:halt, false}, else: {:cont, Map.put(acc, arg, nil)}
    end)
    |> case do
      false -> false
      _ -> true
    end
  end

  def print(args) do
    args
    |> Enum.map(&as_string/1)
    |> IO.puts()

    nil
  end

  def str([arg]), do: as_string(arg)

  defp as_string(list) when is_list(list) do
    "[" <> (list |> Enum.map(&as_string/1) |> Enum.join(", ")) <> "]"
  end

  defp as_string(x), do: to_string(x)
end
