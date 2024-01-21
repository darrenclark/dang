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
     | "if" , expr , scope , if_rest
     | function_def
     ;

scope = "{" , { stmt } , "}" ;

if_rest = { "else" , "if" , expr , scope } , [ "else" , scope ]

function_def        = "fn" , identifier , "(" , [ function_def_args ] , ")" , scope
function_def_args   = identifier , { "," , identifier }

expr     = term | bin_expr ;
bin_expr = expr , "*" , expr    (* prec = 1 *)
         | expr , "/" , expr    (* prec = 1 *)
         | expr , "+" , expr    (* prec = 0 *)
         | expr , "-" , expr    (* prec = 0 *)
         ;

term = integer_literal
     | identifier
     | paren_expr
     | function_call
     ;

paren_expr = "(" , expr , ")" ;

function_call        = identifier , "(" , [ function_call_args ] , ")"
function_call_args   = expr , { "," , expr }

identifier      = ? identifier ? ;
integer_literal = ? digits ? ;
```
