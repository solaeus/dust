module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  extras: $ => [ /\s/, $._comment ],

  conflicts: $ => [
    [$.map, $.assignment_operator],
  ],

  rules: {
    root: $ => prec(1, repeat1($.statement)),

    _comment: $ => /[#][^#\n]*[#|\n]/,

    block: $ => seq(
      optional('async'),
      '{',
      repeat1($.statement),
      '}',
    ),

    statement: $ => prec.left(seq(
      choice(
        $.assignment,
        $.block,
        $.expression,
        $.filter,
        $.find,
        $.for,
        $.if_else,
        $.index_assignment,
        $.insert,
        $.match,
        $.reduce,
        $.remove,
        $.select,
        $.transform,
        $.while,
      ),
      optional(';'),
    )),

    yield: $ => prec.left(seq(
      $.expression,
      '->',
      $.function_call,
    )),
  
    expression: $ => prec.right(choice(
      $._expression_kind,
      seq('(', $._expression_kind, ')'),
    )),

    _expression_kind: $ => choice(
      $.function_call,
      $.identifier,
      $.index,
      $.logic,
      $.math,
      $.value,
      $.yield,
    ),

    _expression_list: $ => repeat1(prec.right(seq(
      $.expression,
      optional(','),
    ))),

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

    integer: $ => token(prec.left(seq(
      optional('-'),
      repeat1(
        choice('1', '2', '3', '4', '5', '6', '7', '8', '9', '0')
      ),
    ))),

    float: $ => token(prec.left(seq(
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

    index_assignment: $ => seq(
      $.index,
      $.assignment_operator,
      $.statement,
    ),

    assignment_operator: $ => choice(
      "=",
      "+=",
      "-=",
    ),

    if_else: $ => prec.right(seq(
      $.if,
      repeat($.else_if),
      optional($.else),
    )),

    if: $ => seq(
      'if',
      $.expression,
      $.block,
    ),

    else_if: $ => seq(
      'else if',
      $.expression,
      $.block,
    ),

    else: $ => seq(
      'else',
      $.block,
    ),

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
      choice(
        'for',
        'async for',
      ),
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
      field('item_id', $.identifier),
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
      $.identifier_list,
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
 
    identifier_list: $ => prec.right(choice(
      seq(
        '|',
        repeat(seq($.identifier, optional(','))),
        '|',
      ),
    )),

    table: $ => prec.right(seq(
      'table',
      $.identifier_list,
      $.expression,
    )),

    function: $ => seq(
      field('parameters', optional($.identifier_list)),
      '=>',
      field('body', $.block),
    ),

    function_call: $ => prec.right(seq(
      '(',
      choice(
        $.built_in_function,
        $._context_defined_function,
      ),
      ')',
    )),

    _context_defined_function: $ => prec.right(1, seq(
      $.identifier,
      optional($._expression_list),
    )),

    built_in_function: $ => prec.right(seq(
      $._built_in_function_name,
      optional($._expression_list),
    )),

    _built_in_function_name: $ => choice(
      // General
      'assert',
      'assert_equal',
      'context',
      'download',
      'help',
      'length',
      'output',
      'output_error',
      'type',

      // Filesystem
      'append',
      'metadata',
      'move',
      'read',
      'workdir',
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