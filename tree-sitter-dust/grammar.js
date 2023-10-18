module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  rules: {
    root: $ => repeat1($.item),

    item: $ => prec.left(repeat1($.statement)),

    statement: $ => choice(
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
    ),
  
    comment: $ => seq(/[#]+.*/),

    expression: $ => choice(
      $._expression_kind,
      seq('(', $._expression_kind, ')'),
    ),

    _expression_kind: $ => prec.right(choice(
      $.value,
      $.identifier,
      $.function_call,
      $.math,
      $.logic,
    )),

    identifier: $ => /[a-z|_|.]+[0-9]?/,

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

    integer: $ => /[-]?[0-9]+/,

    float: $ => /[-]?[0-9]+[.]{1}[0-9]*/,

    string: $ => /("[^"]*?")|('[^']*?')|(`[^`]*?`)/,

    boolean: $ => choice(
      'true',
      'false',
    ),

    list: $ => seq(
      '[',
      repeat(seq($.expression, optional(','))),
      ']'
    ),

    function: $ => seq(
      'function',
      optional(seq('<', repeat(seq($.identifier, optional(','))), '>')),
      '{',
      $.item,
      '}',
    ),

    table: $ => seq(
      'table',
      seq('<', repeat1(seq($.identifier, optional(','))), '>'),
      '{',
      repeat($.expression),
      '}',
    ),

    map: $ => seq(
      '{',
      repeat(seq($.identifier, "=", $.expression)),
      '}',
    ),

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
      optional(seq($.else)),
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

    function_call: $ => prec.right(seq(
      '(',
      choice($.identifier, $.tool),
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

    tool: $ => choice(
      'assert',
      'assert_equal',
      'output',

      'read',
      'write',

      'raw',
      'sh',
      'bash',
      'fish',
      'zsh',

      'random',
      'random_boolean',
      'random_float',
      'random_string',
      'random_integer',

      'length',
      'sort',
      'transform',
      'filter',

      'to_csv',
      'from_csv',
      'to_json',
      'from_json',

      'help',
    ),

    select: $ => prec.right(seq(
      'select',
      $.identifier,
      'from',
      $.identifier,
      optional(
        seq('where', $.expression)
      ),
    )),

    insert: $ => prec.right(seq(
      'insert',
      repeat1($.list),
      'into',
      $.identifier,
      optional(
        seq('where', $.logic)
      ),
    )),

    async: $ => seq(
      'async', 
      '{', 
      repeat($.statement), 
      '}'
    ),
  }
});