; Keywords

[
  "local"
  "while"
  "repeat"
  "until"
  "for"
  "in"
  "if"
  "elseif"
  "else"
  "then"
  "do"
  "function"
  "end"
  "return"
  (continue_statement)
  (break_statement)
] @keyword

(type_alias_declaration
  [
    "export"
    "type"
  ] @keyword)

(type_function_declaration
  [
    "export"
    "type"
  ] @keyword)

(declare_global_declaration
  "declare" @keyword)

(declare_global_function_declaration
  "declare" @keyword)

(declare_class_declaration
  [
    "declare"
    "class"
    "extends"
  ] @keyword)

; Punctuations

[
    "("
    ")"
    "["
    "]"
    "{"
    "}"
    "<"
    ">"
] @punctuation.bracket

[
    ";"
    ":"
    ","
    "."
    "->"
] @punctuation.delimiter

; Operators

(binary_expression
  [
    "<"
    ">"
  ] @operator.comparison)

[
  "=="
  "~="
  "<="
  ">="
] @operator.comparison

[
  "not"
  "and"
  "or"
] @operator.logical

[
  "="
  "+="
  "-="
  "*="
  "/="
  "//="
  "%="
  "^="
  "..="
] @operator.assignment

[
  "+"
  "-"
  "*"
  "/"
  "//"
  "%"
  "^"
] @operator.arithmetic

[
  "#"
  "&"
  "|"
  "::"
  ".."
  "?"
] @operator

; Variables

(identifier) @variable

(string_interpolation
  [
    "{"
    "}"
  ] @punctuation.special) @embedded

(type_binding
  (identifier) @variable.parameter)

((identifier) @variable.special
  (#any-of? @variable.special "math" "table" "coroutine" "bit32" "utf8" "os" "debug" "buffer"
    "vector"))

((identifier) @variable.special
  (#match? @variable.special "^_[A-Z]*$"))

; Tables

(table_constructor
  [
    "{"
    "}"
  ] @constructor)

(field_identifier) @property

; Constants

(nil) @constant.builtin

((identifier) @constant.builtin
  (#eq? @constant.builtin "_VERSION"))

(
  [
    (identifier)
    (field_identifier)
  ] @constant
  (#match? @constant "^[A-Z][A-Z][A-Z_0-9]*$"))

; Literals

(number) @number

[
  (true)
  (false)
] @boolean

(string) @string
(escape_sequence) @string.escape

(string_interpolation
  [
    "{"
    "}"
  ] @punctuation.special)

(interpolated_string
  [
    "`"
  ] @string)

(string_content) @string

; Types

(table_property_attribute) @attribute

(typeof_type
  "typeof" @function.builtin)

(type_identifier) @type

(type_reference
  prefix: (identifier) @variable.namespace)

(type_reference
  prefix: (identifier) @constant.namespace
  (#match? @constant.namespace "^[A-Z][A-Z][A-Z_0-9]*$"))

; Functions

(function_declaration
  name: [
    (identifier) @function
    (dot_index_expression
      field: (field_identifier) @function)
  ])

(method_index_expression
  method: (field_identifier) @function)

(local_function_declaration
  name: (identifier) @function)

(declare_global_function_declaration
  name: (identifier) @function)

(class_function
  name: (identifier) @function)

(parameters
  [
    (binding
      name: (identifier) @variable.parameter)
    (variadic_parameter "..." @variable.parameter)])

((identifier) @variable.special
  (#eq? @variable.special "self"))

(function_call
  name: [
    (identifier) @function
    (dot_index_expression
      field: (field_identifier) @function)
  ])

(parameter_attribute
   name: (identifier) @attribute)

(attribute
  [
    "@" @attribute
    name: (identifier) @attribute
  ])

; require(""), (require)("")
(function_call
  name: [
    (identifier) @function.builtin
    (parenthesized_expression
      (identifier) @function.builtin)
  ]
  (#any-of? @function.builtin
    "require" "assert" "error" "gcinfo" "getfenv" "getmetatable" "next"
    "newproxy" "print" "rawequal" "rawget" "select" "setfenv" "setmetatable"
    "tonumber" "tostring" "type" "typeof" "ipairs" "pairs" "pcall" "xpcall"
    "unpack"))

; tbl.__index, tbl:__index
(function_call
  name: [
    (dot_index_expression
      field: (field_identifier) @function.builtin)
    (method_index_expression
      method: (field_identifier) @function.builtin)
  ]
  (#any-of? @function.builtin
    "__index" "__newindex" "__call" "__concat" "__unm" "__add" "__sub" "__mul"
    "__div" "__idiv" "__mod" "__pow" "__tostring" "__metatable" "__eq" "__lt"
    "__le" "__mode" "__gc" "__len" "__iter"))

(function_call
  name: (dot_index_expression
    table: (identifier) @variable.special
    field: (field_identifier) @function.builtin)
  (#any-of? @variable.special "math" "table" "string" "coroutine" "bit32" "utf8" "os" "debug"
    "buffer" "vector"))

; string.match
(function_call
  name: (dot_index_expression
    table: (identifier) @variable.special
    (#eq? @variable.special "string")
    field: (field_identifier) @function.builtin
    (#any-of? @function.builtin "find" "match" "gmatch" "gsub"))
  arguments: (arguments
    (string)
    (string
      content: (_) @string.regex) @string.regex))

; ("string"):match
(function_call
  name: (method_index_expression
    method: (field_identifier) @function.builtin
    (#any-of? @function.builtin "find" "match" "gmatch" "gsub"))
  arguments: (arguments
    (string
      content: (_) @string.regex) @string.regex))

; TODO: Special highlight for type methods and the `types` library in type functions when it's possible
; to query descendants.

; Comments

(comment) @comment

(hash_bang_line) @preproc

((comment) @comment.doc
  (#match? @comment.doc "^[-][-][-]"))

((comment) @comment.doc
  (#match? @comment.doc "^[-][-](%s?)@"))
