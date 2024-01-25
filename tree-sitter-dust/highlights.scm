(expression) @expression
(value) @value
(identifier) @variable
(value) @value
(string) @string

[
      (integer)
      (float)
] @number

(function) @function

(boolean) @boolean
(list) @list

["," ":" ";"] @punctuation.delimiter

[
      "["
      "]"
      "{"
      "}"
      "<"
      ">"
      "("
      ")"
] @punctuation.bracket

[
      (type)
      (type_specification)
] @type

(assignment_operator) @operator.assignment
(logic_operator) @operator.logic
(math_operator) @operator.math

[
      "async"
      "else"
      "else if"
      "false"
      "for"
      "if"
      "in"
      "match"
      "self"
      "true"
      "while"
      "->"
      "=>"
] @keyword

(built_in_function) @function.builtin
(function_call) @function.call
