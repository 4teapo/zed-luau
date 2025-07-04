# zed-luau
A [Zed](https://zed.dev/) extension that adds support for the [Luau programming language](https://luau.org/).

## Installation
To install zed-luau, you can use the extension menu in Zed, or clone the repository and install it
as a dev extension with `zed: install dev extension`.

## Configuration
This extension can be configured via your Zed `settings.json`. The default configuration looks like
this:

<!-- We limit line lengths to about 80 characters to prevent needing sideways scroll in desktop GitHub-->
```jsonc
{
  "lsp": {
    "luau-lsp": {
      "settings": {
        // luau-lsp settings. What belongs here is specified below this block.
        "luau-lsp": {},
        "roblox": {
          // Whether or not Roblox-specific features should be enabled.
          "enabled": false,
          // The security level of scripts.
          // Must be "roblox_script", "local_user", "plugin" or "none".
          "security_level": "plugin"
        },
        "fflags": {
          // Whether or not all boolean, non-experimental fflags should be enabled
          // by default.
          "enable_by_default": false,
          // Whether or not the new Luau type solver should be enabled.
          "enable_new_solver": false,
          // Whether or not FFlag values should be synced with Roblox's default
          // FFlag values.
          "sync": true,
          // FFlags that are forced to some value.
          "override": {}
        },
        "binary": {
          // Whether or not the extension should skip searching for a binary in
          // your `$PATH` to use instead of installing one itself.
          "ignore_system_version": false,
          // The path to the language server binary you want to force the extension
          // to use.
          "path": null,
          // Additional arguments to pass to the language server. If you want to
          // set exactly which arguments are passed, use `lsp.luau-lsp.binary.path`
          // & `lsp.luau-lsp.binary.args` instead.
          "args": []
        },
        "plugin": {
          // Whether or not Roblox Studio Plugin support should be enabled. If false, the
          // extension will use the regular language server binary only, whereas if true,
          // it will use, thereby starting an HTTP server, and potentially install
          // 4teapo/luau-lsp-proxy as well. This is necessary for plugin support
          // to be possible.
          "enabled": false,
          // The port number to connect the Roblox Studio Plugin to.
          "port": 3667,
          // The path to the luau-lsp-proxy binary you want to force the extension
          // to use. If null, the extension tries to install it itself.
          "proxy_path": null
        },
        // Additional definition file paths to pass to the language server.
        "definitions": [],
        // Additional documentation file paths to pass to the language server.
        "documentation": []
      }
    }
  }
}
```

The configuration options for `settings.luau-lsp` are shown in the `ClientConfiguration` structure
[here](https://github.com/JohnnyMorganz/luau-lsp/blob/main/src/include/LSP/ClientConfiguration.hpp).
For example, to enable inlay hints, you can add the following to your Zed `settings.json`:

```jsonc
{
  "inlay_hints": {
    "enabled": true
  },
  "lsp": {
    "luau-lsp": {
      "settings": {
        "luau-lsp": {
          "inlayHints": {
            "parameterNames": "all"
          }
        }
      }
    }
  }
}
```

## Rojo
zed-luau does not provide Rojo support by itself. It's ergonomical to use [Zed tasks](https://zed.dev/docs/tasks)
to run Rojo commands. For example:

```json
[
  {
    "label": "Serve and autogenerate sourcemap",
    "command": "rojo serve & rojo sourcemap --watch --include-non-scripts --output sourcemap.json"
  },
  {
    "label": "Build and open",
    "command": "rojo build --output a.rbxl; open a.rbxl"
  }
]
```

To get autocompletion in `project.json` files, you can add the following to your Zed `settings.json`:

```jsonc
{
  "lsp": {
    "json-language-server": {
      "settings": {
        "json": {
          "schemas": [
            {
              "fileMatch": ["*.project.json"],
              "url": "https://raw.githubusercontent.com/rojo-rbx/vscode-rojo/refs/heads/master/schemas/project.template.schema.json"
            }
          ]
        }
      }
    }
  }
}
```

## Lune
To use zed-luau with lune, follow the [Editor Setup guide](https://lune-org.github.io/docs/getting-started/4-editor-setup).
The editor settings for Zed are as follows:

```jsonc
{
  "lsp": {
    "luau-lsp": {
      "settings": {
        "luau-lsp": {
          "require": {
            "mode": "relativeToFile",
            "directoryAliases": {
              "@lune/": "~/.lune/.typedefs/x.y.z/"
            }
          }
        }
      }
    }
  }
}
```

## Formatting
For automatically formatting your code, you can install
[StyLua](https://github.com/JohnnyMorganz/StyLua), a Lua code formatter. Then,
use the following settings in your Zed `settings.json`:

```jsonc
{
  "languages": {
    "Luau": {
      "formatter": {
        "external": {
          "command": "stylua",
          "arguments": ["-"]
        }
      }
    }
  }
}
```

## Runnables
zed-luau marks expressions of the form
```luau
x("", ...)
```
where `x` is `it`, `describe`, or `test`, as runnables with the tag `luau-jest-test`, and sets
`$ZED_CUSTOM_script` to the contents of the string parameter. This is helpful if you're using
[jest-lua](https://github.com/jsdotlua/jest-lua) or a similar testing framework.

## Missing Features
- Bytecode generation ([#20042](https://github.com/zed-industries/zed/issues/20042))
- Language-server-assisted end autocomplete
