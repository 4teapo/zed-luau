(comment) @annotation

(local_variable_declaration
  "local" @context
  (binding_list
    (binding
      name: (identifier) @name
      (#match? @name "^[A-Z][A-Z][A-Z_0-9]*$")) @item))

(type_alias_declaration
  "export"? @context
  "type" @context
  name: (type_identifier) @name) @item

(type_alias_declaration
  type: (table_type
    (table_property_list
      [
        (table_property
          attribute: (table_property_attribute)? @context
          left: (field_identifier) @name)?
        (table_indexer
          attribute: (table_property_attribute)? @context
          "[" @context
          (_) @name
          "]" @context)?
      ] @item)))

(type_function_declaration
  "export"? @context
  "type" @context
  "function" @context
  name: (type_identifier) @name
  (parameters
    "(" @context
    ")" @context)) @item

(function_declaration
  "function" @context
  name: (_) @name
  (parameters
    "(" @context
    ")" @context)) @item

(local_function_declaration
  "local" @context
  "function" @context
  name: (_) @name
  (parameters
    "(" @context
    ")" @context)) @item

(chunk
  (return_statement
    "return" @name
    (expression_list)? @context) @item)

(declare_global_declaration
  "declare" @context
  name: (identifier) @name) @item

(declare_global_declaration
  type: (table_type
    (table_property_list
      [
        (table_property
          attribute: (table_property_attribute)? @context
          left: (field_identifier) @name)?
        (table_indexer
          attribute: (table_property_attribute)? @context
          "[" @context
          (_) @name
          "]" @context)?
      ] @item)))

(declare_global_function_declaration
  "declare" @context
  "function" @context
  name: (identifier) @name
  (parameters
    "(" @context
    ")" @context)) @item

(declare_class_declaration
  "declare" @context
  "class" @context
  name: (identifier) @name
  "extends"? @context
  superclass: (identifier)? @name) @item

(class_property
  left: (field_identifier) @name) @item

(class_indexer
  "[" @context
  (_) @name
  "]" @context) @item

(class_function
  "function" @context
  name: (identifier) @name
  (parameters
    "(" @context
    ")" @context)) @item
