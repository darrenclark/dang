# Grammar

```ebnf
program = "return" , expr , ";" ;

expr     = term | bin_expr ;
bin_expr = expr , "*" , expr    (* prec = 1 *)
         | expr , "/" , expr    (* prec = 1 *)
         | expr , "+" , expr    (* prec = 0 *)
         | expr , "-" , expr    (* prec = 0 *)

term = integer_literal ;

integer_literal = ? digits ? ;
```
