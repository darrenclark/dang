# Grammar

```ebnf
program = { stmt } ;

stmt = "return" , expr , ";" ;

expr     = term | bin_expr ;
bin_expr = expr , "*" , expr    (* prec = 1 *)
         | expr , "/" , expr    (* prec = 1 *)
         | expr , "+" , expr    (* prec = 0 *)
         | expr , "-" , expr    (* prec = 0 *)
         ;

term = integer_literal
     | paren_expr
     ;

paren_expr = "(" , expr , ")" ;

integer_literal = ? digits ? ;
```
