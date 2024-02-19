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


"as" @keyword
"async" @keyword
"break" @keyword
"else" @keyword
"else if" @keyword
"enum" @keyword
"false" @keyword
"for" @keyword
"if" @keyword
"in" @keyword
"match" @keyword
"return" @keyword
"struct" @keyword
"true" @keyword
"while" @keyword
"->" @keyword
":" @keyword
"::" @keyword
"^" @keyword
"loop" @keyword

(function_call) @function.call
