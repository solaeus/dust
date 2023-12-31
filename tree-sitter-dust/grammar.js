module.exports = grammar({
  name: 'dust',

  word: $ => $._identifier_pattern,

  extras: $ => [/\s/, $._comment],

  rules: {
    root: $ =>
      prec(1, repeat1($.statement)),

    _comment: $ => /[#][^#\n]*[#|\n]/,

    statement: $ =>
      prec.left(
        seq(
          choice(
            $.assignment,
            $.block,
            $.expression,
            $.for,
            $.if_else,
            $.index_assignment,
            $.match,
            $.return,
            $.while,
          ),
          optional(';'),
        ),
      ),

    expression: $ =>
      choice(
        $._expression_kind,
        seq(
          '(',
          $._expression_kind,
          ')',
        ),
      ),

    _expression_kind: $ =>
      prec.right(
        choice(
          $.function_call,
          $.identifier,
          $.index,
          $.logic,
          $.math,
          $.value,
          $.yield,
        ),
      ),

    _expression_list: $ =>
      repeat1(
        prec.right(
          seq(
            $.expression,
            optional(','),
          ),
        ),
      ),

    block: $ =>
      seq(
        optional('async'),
        '{',
        repeat($.statement),
        '}',
      ),

    identifier: $ =>
      choice(
        $._identifier_pattern,
        $.built_in_function,
      ),

    _identifier_pattern: $ =>
      /[_a-zA-Z]+[_a-zA-Z0-9]*[_a-zA-Z]?/,

    value: $ =>
      choice(
        $.function,
        $.integer,
        $.float,
        $.string,
        $.boolean,
        $.list,
        $.map,
        $.option,
      ),

    integer: $ =>
      token(
        prec.left(
          seq(
            optional('-'),
            repeat1(
              choice(
                '1',
                '2',
                '3',
                '4',
                '5',
                '6',
                '7',
                '8',
                '9',
                '0',
              ),
            ),
          ),
        ),
      ),

    float: $ =>
      token(
        prec.left(
          seq(
            optional('-'),
            repeat1(
              choice(
                '1',
                '2',
                '3',
                '4',
                '5',
                '6',
                '7',
                '8',
                '9',
                '0',
              ),
            ),
            '.',
            repeat1(
              choice(
                '1',
                '2',
                '3',
                '4',
                '5',
                '6',
                '7',
                '8',
                '9',
                '0',
              ),
            ),
          ),
        ),
      ),

    string: $ =>
      /("[^"]*?")|('[^']*?')|(`[^`]*?`)/,

    boolean: $ =>
      choice('true', 'false'),

    list: $ =>
      seq(
        '[',
        repeat(
          prec.left(
            seq(
              $.expression,
              optional(','),
            ),
          ),
        ),
        ']',
      ),

    map: $ =>
      seq(
        '{',
        repeat1(
          seq(
            $.identifier,
            optional($.type_definition),
            '=',
            $.statement,
            optional(','),
          ),
        ),
        '}',
      ),

    option: $ =>
      choice(
        'none',
        seq(
          'some',
          '(',
          $.expression,
          ')',
        ),
      ),

    index: $ =>
      prec.left(
        2,
        seq(
          $.expression,
          ':',
          $.expression,
          optional(
            seq('..', $.expression),
          ),
        ),
      ),

    math: $ =>
      prec.left(
        seq(
          $.expression,
          $.math_operator,
          $.expression,
        ),
      ),

    math_operator: $ =>
      choice('+', '-', '*', '/', '%'),

    logic: $ =>
      prec.left(
        seq(
          $.expression,
          $.logic_operator,
          $.expression,
        ),
      ),

    logic_operator: $ =>
      prec.left(
        choice(
          '==',
          '!=',
          '&&',
          '||',
          '>',
          '<',
          '>=',
          '<=',
        ),
      ),

    assignment: $ =>
      seq(
        $.identifier,
        optional($.type_definition),
        $.assignment_operator,
        $.statement,
      ),

    index_assignment: $ =>
      seq(
        $.index,
        $.assignment_operator,
        $.statement,
      ),

    assignment_operator: $ =>
      prec.right(
        choice('=', '+=', '-='),
      ),

    if_else: $ =>
      prec.right(
        seq(
          $.if,
          repeat($.else_if),
          optional($.else),
        ),
      ),

    if: $ =>
      seq('if', $.expression, $.block),

    else_if: $ =>
      seq(
        'else if',
        $.expression,
        $.block,
      ),

    else: $ => seq('else', $.block),

    match: $ =>
      prec.right(
        seq(
          'match',
          $.expression,
          '{',
          repeat1(
            seq(
              choice($.expression, '*'),
              '=>',
              $.statement,
              optional(','),
            ),
          ),
          '}',
        ),
      ),

    while: $ =>
      seq(
        'while',
        $.expression,
        $.block,
      ),

    for: $ =>
      seq(
        choice('for', 'async for'),
        $.identifier,
        'in',
        $.expression,
        $.block,
      ),

    return: $ =>
      prec.right(
        seq('return', $.statement),
      ),

    type_definition: $ =>
      seq('<', $.type, '>'),

    type: $ =>
      prec.right(
        choice(
          'any',
          'bool',
          'float',
          'int',
          'map',
          'none',
          'num',
          'str',
          seq('[', $.type, ']'),
          seq(
            '(',
            repeat(
              seq(
                $.type,
                optional(','),
              ),
            ),
            ')',
            optional(seq('->', $.type)),
          ),
          seq(
            'option',
            '(',
            $.type,
            ')',
          ),
        ),
      ),

    function: $ =>
      seq(
        '(',
        repeat(
          seq(
            $.identifier,
            $.type_definition,
            optional(','),
          ),
        ),
        ')',
        $.type_definition,
        $.block,
      ),

    function_expression: $ =>
      prec(
        1,
        choice(
          $.function,
          $.function_call,
          $.identifier,
          $.index,
          $.yield,
        ),
      ),

    function_call: $ =>
      prec.right(
        seq(
          $.function_expression,
          '(',
          optional($._expression_list),
          ')',
        ),
      ),

    yield: $ =>
      prec.left(
        1,
        seq(
          $.expression,
          '->',
          $.function_expression,
          optional(
            seq(
              '(',
              $._expression_list,
              ')',
            ),
          ),
        ),
      ),

    built_in_function: $ =>
      choice(
        'assert',
        'assert_equal',
        'bash',
        'download',
        'either_or',
        'fish',
        'from_json',
        'is_none',
        'is_some',
        'length',
        'metadata',
        'output',
        'output_error',
        'random',
        'random_boolean',
        'random_float',
        'random_integer',
        'read',
        'to_json',
        'write',
      ),
  },
});
