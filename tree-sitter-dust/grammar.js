module.exports = grammar({
  name: 'dust',

  word: $ => $.identifier,

  extras: $ => [/\s/, $._comment],

  rules: {
    root: $ =>
      prec(1, repeat1($.statement)),

    _comment: $ => /[#][^#\n]*[#|\n]/,

    block: $ =>
      seq(
        optional('async'),
        '{',
        repeat1($.statement),
        '}',
      ),

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
            $.use,
            $.while,
          ),
          optional(';'),
        ),
      ),

    expression: $ =>
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

    identifier: $ =>
      /[_a-zA-Z]+[_a-zA-Z0-9]?/,

    value: $ =>
      choice(
        $.integer,
        $.float,
        $.function,
        $.string,
        $.boolean,
        $.list,
        $.map,
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
        repeat(
          seq(
            $.identifier,
            '=',
            $.statement,
            optional(','),
          ),
        ),
        '}',
      ),

    index: $ =>
      prec.left(
        1,
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
        choice(
          seq(
            $.expression,
            $.math_operator,
            $.expression,
          ),
          seq(
            '(',
            $.expression,
            $.math_operator,
            $.expression,
            ')',
          ),
        ),
      ),

    math_operator: $ =>
      choice('+', '-', '*', '/', '%'),

    logic: $ =>
      prec.right(
        choice(
          seq(
            $.expression,
            $.logic_operator,
            $.expression,
          ),
          seq(
            '(',
            $.expression,
            $.logic_operator,
            $.expression,
            ')',
          ),
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
          repeat1(
            seq(
              $.expression,
              '=>',
              $.block,
            ),
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

    return: $ =>
      prec.right(
        seq('return', $.expression),
      ),

    use: $ => seq('use', $.string),

    type_definition: $ =>
      seq('<', $.type, '>'),

    type: $ =>
      prec.right(
        choice(
          'any',
          'bool',
          'float',
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
          'int',
          seq('[', $.type, ']'),
          'map',
          'num',
          'str',
        ),
      ),

    function: $ =>
      seq(
        'fn',
        '|',
        repeat(
          seq(
            $.identifier,
            optional(','),
          ),
        ),
        '|',
        $.block,
      ),

    function_call: $ =>
      prec.right(
        seq(
          '(',
          $.expression,
          optional($._expression_list),
          ')',
        ),
      ),

    yield: $ =>
      prec.left(
        seq(
          $.expression,
          '->',
          '(',
          $.expression,
          optional($._expression_list),
          ')',
        ),
      ),
  },
});
