module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  rules: {
    root: $ => repeat1($.item),

    item: $ => prec.left(repeat1($.statement)),

    statement: $ => prec.left(choice(
      $.comment,
      $.assignment,
      $.expression,
      $.if_else,
      $.insert,
      $.select,
      $.while,
      $.async,
      $.for,
      $.transform,
      $.filter,
      $.find,
      $.remove,
    )),
  
    comment: $ => seq(/[#]+.*/),

    expression: $ => choice(
      $._expression_kind,
      seq('(', $._expression_kind, ')'),
    ),

    _expression_kind: $ => choice(
      $.value,
      $.identifier,
      $.index,
      $.math,
      $.logic,
      $.function_call,
      $.tool,
    ),

    identifier: $ => /[_a-zA-Z]+[_a-zA-Z0-9]?/,

    value: $ => choice(
      $.integer,
      $.float,
      $.string,
      $.boolean,
      $.list,
      $.function,
      $.table,
      $.map,
    ),

    _numeric: $ => token(repeat1(
      choice('1', '2', '3', '4', '5', '6', '7', '8', '9', '0')
    )),

    integer: $ => prec.left(seq(
      optional(token.immediate('-')),
      $._numeric,
    )),

    float: $ => prec.left(seq(
      optional(token.immediate('-')),
      $._numeric,
      token.immediate('.'),
      $._numeric,
    )),

    string: $ => /("[^"]*?")|('[^']*?')|(`[^`]*?`)/,

    boolean: $ => choice(
      'true',
      'false',
    ),

    list: $ => seq(
      '[',
      repeat(seq($.expression, optional(','))),
      ']',
    ),

    map: $ => seq(
      '{',
      repeat(seq($.identifier, "=", $.expression)),
      '}',
    ),

    index: $ => prec.left(seq(
      $.expression,
      ':',
      $.expression,
      optional(seq(
        '..',
        $.expression,
      )),
    )),
 
    function: $ => seq(
      'function',
      optional(seq('<', repeat(seq($.identifier, optional(','))), '>')),
      '{',
      $.item,
      '}',
    ),

    table: $ => prec.left(seq(
      'table',
      seq('<', repeat1(seq($.identifier, optional(','))), '>'),
      $.expression,
    )),

    math: $ => prec.left(seq(
      $.expression,
      $.math_operator,      
      $.expression,
    )),

    math_operator: $ => choice(
      '+',
      '-',
      '*',
      '/',
      '%',
    ),

    logic: $ => prec.right(seq(
      $.expression,
      $.logic_operator,
      $.expression,
    )),

    logic_operator: $ => choice(
      '==',
      '!=',
      '&&',
      '||',
      '>',
      '<',
      ">=",
      "<=",
    ),

    assignment: $ => prec.right(seq(
      $.identifier,
      $.assignment_operator,
      $.statement,
    )),

    assignment_operator: $ => choice(
      "=",
      "+=",
      "-=",
    ),

    if_else: $ => prec.left(seq(
      $.if,
      repeat(seq($.else_if)),
      optional($.else),
    )),

    if: $ => seq(
      'if',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    else_if: $ => seq(
      'else if',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    else: $ => seq(
      'else',
      '{',
      $.statement,
      '}',
    ),

    function_call: $ => prec(1, seq(
      '(',
      $.identifier,
      repeat(seq($.expression, optional(','))),
      ')',
    )),

    while: $ => seq(
      'while',
      $.expression,
      '{',
      $.item,
      '}',      
    ),

    for: $ => seq(
      'for',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.item,
      '}',
    ),

    transform: $ => seq(
      'transform',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.item,
      '}',
    ),

    filter: $ => seq(
      'filter',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.item,
      '}',
    ),

    find: $ => seq(
      'find',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.item,
      '}',
    ),

    remove: $ => seq(
      'remove',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.item,
      '}',
    ),

    select: $ => prec.right(seq(
      'select',
      '<',
      repeat(seq($.identifier, optional(','))),
      '>',
      'from',
      $.expression,
      optional(seq('{', $.item, '}')),
    )),

    insert: $ => prec.right(seq(
      'insert',
      'into',
      $.identifier,
      $.expression,
    )),

    async: $ => seq(
      'async', 
      '{', 
      repeat($.statement), 
      '}'
    ),

    tool: $ => prec.right(seq(
      '(',
      $._tool_kind,
      repeat(seq($.expression, optional(','))),
      ')',
    )),

    _tool_kind: $ => choice(
      // General
      'assert',
      'assert_equal',
      'download',
      'help',
      'length',
      'output',
      'output_error',
      'type',
      'workdir',

      // Filesystem
      'append',
      'metadata',
      'move',
      'read',
      'remove',
      'write',

      // Format conversion
      'from_json',
      'to_json',
      'to_string',
      'to_float',

      // Command
      'bash',
      'fish',
      'raw',
      'sh',
      'zsh',

      // Random
      'random',
      'random_boolean',
      'random_float',
      'random_integer',
      
      // Tables
      'columns',
      'rows',
      
      // Lists
      'reverse',
    ),
  }
});