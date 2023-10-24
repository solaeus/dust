(expression) @expression
(value) @value
(comment) @comment
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
] @punctuation.bracket

[
      (assignment_operator)
      (logic_operator)
      (math_operator)
] @operator

[
      "if"
      "else"
      "for"
      "transform"
      "in"
      "function"
] @keyword

[
      "assert"
      "assert_equal"
      "download"
      "help"
      "length"
      "output"
      "output_error"
      "type"
      "workdir"
      "append"
      "metadata"
      "move"
      "read"
      "remove"
      "write"
      "bash"
      "fish"
      "raw"
      "sh"
      "zsh"
      "random"
      "random_boolean"
      "random_float"
      "random_integer"
      "columns"
      "rows"
] @function.builtin