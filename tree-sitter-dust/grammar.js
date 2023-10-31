module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  extras: $ => [ /\s/, $.comment ],

  conflicts: $ => [
    [$.map, $.assignment_operator],
  ],

  rules: {
    root: $ => repeat1($.block),

    comment: $ => /[#][^#\n]*[#|\n]/,

    block: $ => prec.right(choice(
      repeat1($.statement),
      seq('{', repeat1($.statement), '}'),
    )),

    statement: $ => prec.right(seq(
      $._statement_kind,
      optional(';'),
    )),

    _statement_kind: $ => prec.right(choice(
      $.assignment,
      $.async,
      $.expression,
      $.filter,
      $.find,
      $.for,
      $.if_else,
      $.insert,
      $.match,
      $.reduce,
      $.remove,
      $.select,
      $.transform,
      $.while,
    )),
  
    expression: $ => prec.left(choice(
      $._expression_kind,
      seq('(', $._expression_kind, ')'),
    )),

    _expression_kind: $ => prec.left(1, choice(
      $.function_call,
      $.identifier,
      $.index,
      $.logic,
      $.math,
      $.value,
    )),

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

    integer: $ => prec.left(token(seq(
      optional('-'),
      repeat1(
        choice('1', '2', '3', '4', '5', '6', '7', '8', '9', '0')
      ),
    ))),

    float: $ => prec.left(token(seq(
      optional('-'),
      repeat1(choice('1', '2', '3', '4', '5', '6', '7', '8', '9', '0')),
      '.',
      repeat1(choice('1', '2', '3', '4', '5', '6', '7', '8', '9', '0')),
    ))),

    string: $ => /("[^"]*?")|('[^']*?')|(`[^`]*?`)/,

    boolean: $ => choice(
      'true',
      'false',
    ),

    list: $ => seq(
      '[',
      repeat(prec.left(seq($.expression, optional(',')))),
      ']',
    ),

    map: $ => seq(
      '{',
      repeat(seq(
        $.identifier,
        "=",
        $.statement,
        optional(',')
      )),
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
      $.block,
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

    assignment: $ => seq(
      $.identifier,
      $.assignment_operator,
      $.statement,
    ),

    assignment_operator: $ => choice(
      "=",
      "+=",
      "-=",
    ),

    if_else: $ => prec.left(seq(
      $.if,
      repeat($.else_if),
      optional($.else),
    )),

    if: $ => prec.left(seq(
      'if',
      $.expression,
      $.block,
    )),

    else_if: $ => prec.left(seq(
      'else if',
      $.expression,
      $.block,
    )),

    else: $ => prec.left(seq(
      'else',
      $.block,
    )),

    function_call: $ => prec.right(seq(
      choice($.identifier, $.built_in_function),
      repeat(prec.right(seq($.expression, optional(',')))),
    )),

    match: $ => prec.right(seq(
      'match',
      $.expression,
      repeat1(seq(
        $.expression,
        '=>',
        $.block,
      )),
    )),

    while: $ => seq(
      'while',
      $.expression,
      $.block,
    ),

    for: $ => seq(
      'for',
      $.identifier,
      'in',
      $.expression,
      $.block,
    ),

    transform: $ => seq(
      'transform',
      $.identifier,
      'in',
      $.expression,
      $.block,
    ),

    filter: $ => seq(
      'filter',
      field('count', optional($.expression)),
      field('statement_id', $.identifier),
      'in',
      field('collection', $.expression),
      field('predicate', $.block),
    ),

    find: $ => seq(
      'find',
      $.identifier,
      'in',
      $.expression,
      $.block,
    ),

    remove: $ => seq(
      'remove',
      $.identifier,
      'from',
      $.expression,
      $.block,
    ),

    reduce: $ => seq(
      'reduce',
      $.identifier,
      'to',
      $.identifier,
      'in',
      $.expression,
      $.block,
    ),

    select: $ => prec.right(seq(
      'select',
      '<',
      repeat(seq($.identifier, optional(','))),
      '>',
      'from',
      $.expression,
      optional($.block),
    )),

    insert: $ => prec.right(seq(
      'insert',
      'into',
      $.identifier,
      $.expression,
    )),

    async: $ => seq(
      'async', 
      $.block,
    ),

    built_in_function: $ => choice(
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