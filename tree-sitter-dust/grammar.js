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
          optional(
            choice('return', 'break'),
          ),
          $.statement_kind,
          optional(';'),
        ),
      ),

    statement_kind: $ =>
      prec.left(
        choice(
          $.assignment,
          $.block,
          $.expression,
          $.for,
          $.if_else,
          $.index_assignment,
          $.match,
          $.pipe,
          $.while,
          $.type_definition,
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
        $.range,
        $.struct_instance,
        $.enum_instance,
      ),

    range: $ => /\d+[.]{2}\d+/,

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
          repeat1(
            seq(
              $.match_pattern,
              '->',
              $.statement,
              optional(','),
            ),
          ),
        ),
      ),

    match_pattern: $ =>
      choice(
        $.enum_pattern,
        $.value,
        '*',
      ),

    enum_pattern: $ =>
      prec(
        1,
        seq(
          $.identifier,
          '::',
          $.identifier,
          optional(
            seq('(', $.identifier, ')'),
          ),
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
          seq(
            '{',
            repeat1(
              seq(
                $.identifier,
                $.type_specification,
              ),
            ),
            '}',
          ),
          'none',
          'num',
          'str',
          $.identifier,
          seq(
            $.identifier,
            '<',
            repeat1(
              seq(
                $.type,
                optional(','),
              ),
            ),
            '>',
          ),
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

    type_definition: $ =>
      choice(
        $.enum_definition,
        $.struct_definition,
      ),

    enum_definition: $ =>
      prec.right(
        seq(
          'enum',
          $.identifier,
          repeat(
            seq(
              '{',
              repeat1(
                seq(
                  $.identifier,
                  optional(
                    seq(
                      '(',
                      choice(
                        $.type,
                        $.type_definition,
                      ),
                      ')',
                    ),
                  ),
                  optional(','),
                ),
              ),
              '}',
            ),
          ),
        ),
      ),

    enum_instance: $ =>
      prec.right(
        seq(
          $.identifier,
          '::',
          $.identifier,
          optional(
            seq('(', $.expression, ')'),
          ),
        ),
      ),

    struct_definition: $ =>
      seq(
        'struct',
        $.identifier,
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

    struct_instance: $ =>
      seq($.identifier, '::', $.map),
  },
});
