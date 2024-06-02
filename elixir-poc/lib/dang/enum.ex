defmodule Dang.Enum do
  @moduledoc """
  Describes enumerable types in dang
  """

  defguard is_enum(x) when is_list(x) or is_binary(x)

  def len(x) when is_list(x), do: length(x)
  def len(x) when is_binary(x), do: String.length(x)

  def at(x, i) when is_list(x), do: Enum.at(x, i)
  def at(x, i) when is_binary(x), do: String.at(x, i)
end
