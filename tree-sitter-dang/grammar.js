// Adapted from https://github.com/tree-sitter-grammars/tree-sitter-lua/blob/main/grammar.js

const PREC = {
  OR: 1, // or
  AND: 2, // and
  COMPARE: 3, // < > <= >= ~= ==
  BIT_OR: 4, // |
  BIT_NOT: 5, // ~
  BIT_AND: 6, // &
  BIT_SHIFT: 7, // << >>
  CONCAT: 8, // ..
  PLUS: 9, // + -
  MULTI: 10, // * / // %
  UNARY: 11, // not # - ~
  POWER: 12, // ^
};

const list_seq = (rule, separator, trailing_separator = false) =>
  trailing_separator
    ? seq(rule, repeat(seq(separator, rule)), optional(separator))
    : seq(rule, repeat(seq(separator, rule)));

module.exports = grammar({
  name: 'dang',

  rules: {
    source_file: $ => repeat($._stmt),

    _stmt: $ => choice(
      $._expr
    ),

    _expr: $ => choice(
      $.function_call,
      $.identifier,
      $.string_literal,
      $.number_literal
    ),

    identifier: $ => /[A-Za-z_?]+/,

    function_call: $ => seq(
      field('fun', $.identifier),
      field('arg', $._arguments)
    ),

    _arguments: $ => seq(
      '(',
      optional(list_seq($._expr, ',')),
      ')'
    ),

    string_literal: $ => seq(
      '"',
      repeat(token(prec(-1, /([^"`$\\\r\n]|\\(.|\r?\n))+/))),
      '"'
    ),

    number_literal: $ => /\d+/,

    /*_definition: $ => choice(
      $.let_stmt,
      $.var_stmt,
      $.assignment,
      $.if_stmt,
      $.for_in_stmt,
      $.expression,
      // TODO: more definitions
    ),

    let_stmt: $ => seq(
      'let',
      field('name', $.identifier),
      '=',
      field('value', $.expression)
    ),

    var_stmt: $ => seq(
      'var',
      field('name', $.identifier),
      '=',
      field('value', $.expression)
    ),

    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $.expression)
    ),

    expression: $ => choice(
      $.nil,
      $.identifier,
      $.number,
      $.anonymous_function,
      $.function_call,
      $.binary_expression,
      $.string
      //$.fn
    ),

    nil: $ => 'nil',


    number: $ => /\d+/,




    body: $ => seq(
      '{',
      repeat($._definition),
      '}'
    ),

    anonymous_function: $ => seq(
      'fn',
      $.argument_names,
      $.body
    ),

    argument_names: $ => seq(
      '(',
      optional(list_seq($.identifier, ',')),
      ')'
    ),

    if_stmt: $ => seq(
      'if',
      $.expression,
      $.body
    ),

    for_in_stmt: $ => seq(
      'for',
      $.identifier,
      'in',
      $.expression,
      $.body
    ),

    binary_expression: ($) =>
      choice(
        ...[
          ['&&', PREC.OR],
          ['||', PREC.AND],
          ['<', PREC.COMPARE],
          ['<=', PREC.COMPARE],
          ['==', PREC.COMPARE],
          ['~=', PREC.COMPARE],
          ['>=', PREC.COMPARE],
          ['>', PREC.COMPARE],
          ['|', PREC.BIT_OR],
          ['~', PREC.BIT_NOT],
          ['&', PREC.BIT_AND],
          ['<<', PREC.BIT_SHIFT],
          ['>>', PREC.BIT_SHIFT],
          ['+', PREC.PLUS],
          ['-', PREC.PLUS],
          ['*', PREC.MULTI],
          ['/', PREC.MULTI],
          ['//', PREC.MULTI],
          ['%', PREC.MULTI],
        ].map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field('left', $.expression),
              operator,
              field('right', $.expression)
            )
          )
        ),
        ...[
          ['..', PREC.CONCAT],
          ['^', PREC.POWER],
        ].map(([operator, precedence]) =>
          prec.right(
            precedence,
            seq(
              field('left', $.expression),
              operator,
              field('right', $.expression)
            )
          )
        )
      ),

    //fn: $ => seq(
    //  'fn',
    //  '(',
    */
  }
});
