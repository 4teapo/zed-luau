; Jest Lua test runnable.
(
  (function_call
    name: (identifier) @_name
    (#any-of? @_name "it" "test" "describe")
    arguments: (arguments
        (string
          content: (_) @run @script))) @_luau-jest-test
  (#set! tag luau-jest-test))
