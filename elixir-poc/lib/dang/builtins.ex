defmodule Dang.Builtins do
  def builtins do
    %{
      :+ => {:builtin, :+},
    }
  end

  def call(:+, args) do
    Enum.sum(args)
  end
end
