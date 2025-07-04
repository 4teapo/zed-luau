(function_definition
  body: (_)* @function.inside) @function.around

(function_declaration
  body: (_)* @function.inside) @function.around

(local_function_declaration
  body: (_)* @function.inside) @function.around

(type_alias_declaration
  type: (_)* @class.inside) @class.around

(type_function_declaration
  body: (_)* @function.inside) @function.around

(comment)+ @comment.around
