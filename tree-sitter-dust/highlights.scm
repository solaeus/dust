(expression) @expression
(value) @value
(identifier) @identifier
(value) @value
(string) @string

[
      (integer)
      (float)
] @number

(function) @function

(boolean) @boolean
(list) @list

"," @punctuation.delimiter

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
      (assignment_operator)
      (logic_operator)
      (math_operator)
] @operator

[
      "any"
      "async"
      "else"
      "false"
      "float"
      "for"
      "if"
      "in"
      "int"
      "map"
      "match"
      "num"
      "str"
      "true"
      "while"
      "->"
      "=>"
] @keyword

(built_in_function) @function.builtin
