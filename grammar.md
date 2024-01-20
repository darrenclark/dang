# Grammar

```ebnf
program = { stmt } ;

stmt = "return" , expr , ";"
     | "let" , identifier , "=" , expr , ";"
     | identifier , "=" , expr , ";"
     | scope
     ;

scope = "{" , { stmt } , "}" ;

expr     = term | bin_expr ;
bin_expr = expr , "*" , expr    (* prec = 1 *)
         | expr , "/" , expr    (* prec = 1 *)
         | expr , "+" , expr    (* prec = 0 *)
         | expr , "-" , expr    (* prec = 0 *)
         ;

term = integer_literal
     | identifier
     | paren_expr
     ;

paren_expr = "(" , expr , ")" ;

identifier      = ? identifier ? ;
integer_literal = ? digits ? ;
```
