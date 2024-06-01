defmodule Dang.Parser do
  import NimbleParsec

  ws = ascii_string([?\s, ?\t, ?\r, ?\n], min: 1) |> ignore()
  opt_ws = ascii_string([?\s, ?\t, ?\r, ?\n], min: 0) |> ignore()

  number = integer(min: 1)

  symbols = [?!..?', ?*..?/, ?:..?@, ?|, ?~]

  atom =
    ascii_string([?a..?z, ?A..?Z | symbols], min: 1)
    |> ascii_string([?a..?z, ?A..?Z, ?0..?9, ?+ | symbols], min: 0)
    |> reduce({Enum, :join, []})
    |> map({String, :to_atom, []})

  open_paren = ignore(string("("))
  close_paren = ignore(string(")"))

  quoted_string =
    ignore(ascii_char([?"]))
    |> repeat(
      choice([
        ignore(string("\\")) |> ascii_char([]),
        ascii_char([{:not, ?"}])
      ])
    )
    |> ignore(ascii_char([?"]))
    |> reduce({List, :to_string, []})

  defparsecp :sexpr,
             open_paren
             |> concat(opt_ws)
             |> optional(choice([parsec(:sexpr), quoted_string, number, atom]))
             |> repeat(ws |> choice([parsec(:sexpr), quoted_string, number, atom]))
             |> concat(opt_ws)
             |> concat(close_paren)
             |> wrap()

  defparsecp :file, repeat(opt_ws |> parsec(:sexpr) |> concat(opt_ws))

  def parse(input) do
    {:ok, result, "", _, _, _} = file(input)
    result
  end
end
