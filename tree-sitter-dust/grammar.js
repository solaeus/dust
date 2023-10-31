module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  rules: {
    root: $ => repeat1($.statement),

    statement: $ => prec.left(choice(
      repeat1($._statement_kind),
      seq('{', $._statement_kind, '}'),
    // ))

    _statement_kind: $ => prec.left(choice(
      $.assignment,
      $.async,
      $.comment,
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
  
    comment: $ => seq(/[#]+.*/),

    expression: $ => choice(
      $._expression_kind,
      seq('(', $._expression_kind, ')'),
    ),

    _expression_kind: $ => choice(
      $.function_call,
      $.identifier,
      $.index,
      $.logic,
      $.math,
      $.tool,
      $.value,
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
      repeat(seq($.expression, optional(','))),
      ']',
    ),

    map: $ => seq(
      '{',
      $.assignment,
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
      $.statement,
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

    match: $ => seq(
      'match',
      $.expression,
      '{',
      repeat1(seq(
        $.expression,
        '=>',
        $.statement,
      )),
      '}',
    ),

    while: $ => seq(
      'while',
      $.expression,
      '{',
      $.statement,
      '}',      
    ),

    for: $ => seq(
      'for',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    transform: $ => seq(
      'transform',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    filter: $ => seq(
      'filter',
      field('count', optional($.expression)),
      field('statement_id', $.identifier),
      'in',
      field('collection', $.expression),
      '{',
      field('predicate', $.statement),
      '}',
    ),

    find: $ => seq(
      'find',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    remove: $ => seq(
      'remove',
      $.identifier,
      'from',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    reduce: $ => seq(
      'reduce',
      $.identifier,
      'to',
      $.identifier,
      'in',
      $.expression,
      '{',
      $.statement,
      '}',
    ),

    select: $ => prec.right(seq(
      'select',
      '<',
      repeat(seq($.identifier, optional(','))),
      '>',
      'from',
      $.expression,
      optional(seq('{', $.statement, '}')),
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