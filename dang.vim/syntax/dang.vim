" Keywords

syntax keyword dangKeywords
  \ let
  \ var
  \ if
  \ else
  \ for
  \ in
  \ fn
  \ import
  \ module
  \ struct

syntax match dangNumber "\v<\d+>"
syntax match dangNumber "\v<\d+\.\d+>"

syntax keyword dangBoolean
      \ true
      \ false
      \ nil

syntax match dangComment "\v\/\/.*$" oneline

syntax region dangString start=/"/ skip=/\\\\\|\\"/ end=/"/

syntax match dangOperator "\v\=+"
syntax match dangOperator "\v\&\&"
syntax match dangOperator "\v\|\|"
syntax match dangOperator "\v\>\=?"
syntax match dangOperator "\v\<\=?"
syntax match dangOperator "\v\+"
syntax match dangOperator "\v\-"
syntax match dangOperator "\v\*"
syntax match dangOperator "\v\/\/@!"

syntax match dangFunction "\<\k\+\ze("


" Highlights
highlight default link dangKeywords Keyword
highlight default link dangNumber Number
highlight default link dangBoolean Number
highlight default link dangComment Comment
highlight default link dangString String
highlight default link dangOperator Operator
highlight default link dangFunction Function
