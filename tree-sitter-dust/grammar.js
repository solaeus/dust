module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

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
            $.pipe,
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
          $.as,
          $.function_call,
          $.identifier,
          $.index,
          $.logic,
          $.math,
          $.value,
          $.yield,
          $.new,
          $.command,
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

    as: $ =>
      seq($.expression, 'as', $.type),

    pipe: $ =>
      prec(
        1,
        seq(
          choice(
            $.command,
            $.function_call,
          ),
          '|',
          choice(
            $.command,
            $.pipe,
            $.function_call,
          ),
        ),
      ),

    command: $ =>
      prec.right(
        seq(
          '^',
          $.command_text,
          repeat($.command_argument),
        ),
      ),

    command_text: $ => /\S+/,

    command_argument: $ =>
      choice(
        /[^^|;\s]+/,
        /("[^"]*?")|('[^']*?')|(`[^`]*?`)/,
      ),

    block: $ =>
      seq(
        optional('async'),
        '{',
        repeat($.statement),
        '}',
      ),

    identifier: $ =>
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
        $.built_in_value,
        $.structure,
        $.range,
      ),

    range: $ => /\d+[.]{2}\d+/,

    structure: $ =>
      seq(
        'struct',
        '{',
        repeat(
          choice(
            seq(
              $.identifier,
              $.type_specification,
            ),
            seq(
              $.identifier,
              '=',
              $.statement,
            ),
            seq(
              $.identifier,
              $.type_specification,
              '=',
              $.statement,
            ),
          ),
        ),
        '}',
      ),

    new: $ =>
      seq(
        'new',
        $.identifier,
        '{',
        repeat(
          choice(
            seq(
              $.identifier,
              '=',
              $.statement,
            ),
            seq(
              $.identifier,
              $.type_specification,
              '=',
              $.statement,
            ),
          ),
        ),
        '}',
      ),

    integer: $ => /[-]?\d+/,

    float: $ =>
      choice(
        /[-|+]?\d*[.][\d|e|-]*/,
        'Infinity',
        'infinity',
        'NaN',
        'nan',
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
      prec(
        1,
        seq(
          '{',
          repeat(
            seq(
              $.identifier,
              optional(
                $.type_specification,
              ),
              '=',
              $.statement,
              optional(','),
            ),
          ),
          '}',
        ),
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
        1,
        seq(
          $.index_expression,
          ':',
          $.index_expression,
        ),
      ),

    index_expression: $ =>
      prec(
        1,
        choice(
          seq(
            '(',
            $.function_call,
            ')',
          ),
          $.identifier,
          $.index,
          $.value,
          $.range,
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
        optional($.type_specification),
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

    type_specification: $ =>
      seq('<', $.type, '>'),

    type: $ =>
      prec.right(
        choice(
          'any',
          'bool',
          'collection',
          'float',
          'int',
          'map',
          'none',
          'num',
          'str',
          $.identifier,
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
            $.type_specification,
            optional(','),
          ),
        ),
        ')',
        $.type_specification,
        $.block,
      ),

    function_expression: $ =>
      choice(
        $._function_expression_kind,
        seq(
          '(',
          $._function_expression_kind,
          ')',
        ),
      ),

    _function_expression_kind: $ =>
      prec(
        2,
        choice(
          $.function_call,
          $.identifier,
          $.index,
          $.value,
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

    built_in_value: $ =>
      choice(
        'args',
        'assert_equal',
        'env',
        'fs',
        'json',
        'length',
        'output',
        'random',
        'str',
      ),
  },
});
