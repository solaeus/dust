(statement) @statement
[
      (expression)
      (function_expression)
      (index_expression)
] @expression
(value) @value
(identifier) @variable

(value) @value
(string) @string
[
      (integer)
      (float)
] @number
[
      (command)
      (function)
] @function
(range) @range
(boolean) @boolean
(list) @list
(map) @map

(struct_definition) @struct
(enum_definition) @enum
 
(block) @block

["," ";"] @punctuation.delimiter

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

(type) @type

(assignment_operator) @operator.assignment
(logic_operator) @operator.logic
(math_operator) @operator.math

[
      "as"
      "async"
      "break"
      "else"
      "else if"
      "enum"
      "false"
      "for"
      "if"
      "in"
      "loop"
      "match"
      "return"
      "struct"
      "true"
      "while"
      "->"
      ":"
      "::"
      "^"
] @keyword

(function_call) @function.call
