defmodule Dang.Env do
  alias __MODULE__

  defstruct globals: %{},
            locals: %{},
            scope: :global

  def new(bindings \\ []), do: %Dang.Env{globals: Map.new(bindings)}

  def put(%Env{scope: :global} = env, key, value) do
    %{env | globals: Map.put(env.globals, key, value)}
  end

  def put(%Env{scope: :local} = env, key, value) do
    %{env | locals: Map.put(env.locals, key, value)}
  end

  def fetch!(%Env{} = env, key) do
    case env do
      %{locals: %{^key => val}} -> val
      %{globals: %{^key => val}} -> val
      _ -> raise "Variable #{inspect(key)} not found"
    end
  end

  def capture(%Env{} = env) do
    %{env | globals: %{}, scope: :local}
  end

  def function_env(%Env{} = env, %Env{} = captured_env, args) do
    %{captured_env |
      globals: env.globals,
      locals: Map.merge(captured_env.locals, args)}
  end
end
