# Grammar

- Whitespace is not significant (and is not described below)
- C style comments are supported (not described below)
    - `// comment here`
    - `/* comment here */`

```ebnf
program = { stmt } ;

stmt = "return" , expr , ";"
     | "let" , identifier , "=" , expr , ";"
     | identifier , "=" , expr , ";"
     | scope
     | "if" , expr , scope , if rest
     ;

scope = "{" , { stmt } , "}" ;

if rest = { "else" , "if" , expr , scope } , [ "else" , scope ]

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
