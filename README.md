# zed-luau
A [Zed](https://zed.dev/) extension that adds support for the [Luau programming language](https://luau.org/).

## Features
- [x] Syntax highlighting
- [x] Outline support
- [x] Runnables
- [x] Non-optional luau-lsp features (excluding end autocomplete. See [#19788](https://github.com/zed-industries/zed/issues/19788))
- [x] Roblox documentation and definitions
- [x] Managing documentation and definitions
- [x] Managing FFlags
- [ ] Luau LSP plugin support (requires additions to Zed's extension API. See [#20042](https://github.com/zed-industries/zed/issues/20042) & [#20040](https://github.com/zed-industries/zed/issues/20040))
- [ ] Bytecode generation (requires additions to Zed's extension API. See [#20042](https://github.com/zed-industries/zed/issues/20042))

## Installation
To install zed-luau, you can use the extension menu in Zed, or clone the repository and install it
as a dev extension with `zed: install dev extension`.

## Support
The latest version of zed-luau supports Zed 0.161.1 and up.

## Configuration
This extension can be configured via your Zed `settings.json`. The default configuration looks like
this:

```jsonc
{
  "lsp": {
    "luau-lsp": {
      "settings": {
        // luau-lsp settings. These are read by luau-lsp itself.
        "luau-lsp": {},
        // Extension settings. These are read by the extension itself.
        "ext": {
          "roblox": {
            // Whether or not Roblox-specific features should be enabled.
            "enabled": false,
            // The security level of scripts.
            // Must be "roblox_script", "local_user", "plugin" or "none".
            "security_level": "plugin"
          },
          "fflags": {
            // Whether or not all boolean, non-experimental fflags should be
            // enabled by default.
            "enable_by_default": false,
            // Whether or not FFlag values should be synced with Roblox's
            // default FFlag values.
            "sync": true,
            // FFlags that are forced to some value.
            "override": {}
          },
          "binary": {
            // Whether or not the extension should skip searching for a binary in your `$PATH` to
            // use instead of installing one itself.
            "ignore_system_version": false,
            // The path to the language server binary you want to force the extension to use.
            "path": null,
            // Additional arguments to pass to the language server. If you want to set exactly which
            // arguments are passed, use `lsp.luau-lsp.binary.path` & `lsp.luau-lsp.binary.args` instead.
            "args": []
          },
          // Additional definition files to pass to the language server.
          // On Windows, the paths are interpreted as absolute if and only if they contain ':'.
          // On other platforms, they're interpreted as absolute if and only if they begin with '/'.
          // Relative paths are relative to the worktree.
          "definitions": [],
          // Additional documentation files to pass to the language server.
          // On Windows, the paths are interpreted as absolute if and only if they contain ':'.
          // On other platforms, they're interpreted as absolute if and only if they begin with '/'.
          // Relative paths are relative to the worktree.
          "documentation": []
        }
      }
    }
  }
}
```

The configuration options for `settings.luau-lsp` can be viewed
[here](https://github.com/JohnnyMorganz/luau-lsp/blob/ae63ce5e10bc5d42122669fc20606fc5ec2fe54d/src/include/LSP/ClientConfiguration.hpp#L220).
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

## Formatting
For automatically formatting your code, install
[StyLua](https://github.com/JohnnyMorganz/StyLua), a Lua code formatter. Then,
add the following to your Zed `settings.json`:

```jsonc
{
  "languages": {
    "Luau": {
      "formatter": {
        "external": {
          "command": "stylua",
          "arguments": ["--stdin-filepath", "{buffer_path}", "-"]
        }
      }
    }
  }
}
```

## Troubleshooting
Syntax highlighting issues stem from problems in the syntax tree, which can be viewed with `debug: open syntax tree view`.
Report syntax tree issues to the [4teapo/tree-sitter-luau](https://github.com/4teapo/tree-sitter-luau).

If zed-luau emitted an error, you will find it in `zed: open log`, and you can view the output of `luau-lsp`
as well as communication between the extension and the language server with `zed: open language server logs`.

## Runnables
zed-luau marks expressions of the form
```luau
x("", ...)
```
where `x` is `it`, `describe`, or `test`, as runnables with the tag `luau-jest-test`, and sets
`$ZED_CUSTOM_script` to the contents of the string parameter. This is helpful if you're using
[jest-lua](https://github.com/jsdotlua/jest-lua) or a similar testing framework.

## FAQ
### How do I use [Rojo](https://rojo.space/) in Zed?
You can add something along the lines of the following to your [Zed tasks](https://zed.dev/docs/tasks):

```json
[
  {
    "label": "Rojo autogenerate sourcemap",
    "command": "rojo sourcemap --watch --output sourcemap.json --include-non-scripts",
  },
  {
    "label": "Rojo serve default.project.json",
    "command": "rojo serve default.project.json",
  },
  {
    "label": "Rojo build out.rbxl",
    "command": "rojo build --output out.rbxl"
  }
]
```

### How do I use zed-luau for `.lua` files as well?
By making Luau your preferred language for `.lua` files in your Zed `settings.json`:

```jsonc
{
  "file_types": {
    "Luau": ["lua"]
  }
}
```

### How do I use a nightly version of luau-lsp?
You need to install the nightly version manually. Afterwards, add it to your PATH and ensure
`binary.ignore_system_version` is set to false, or set `binary.path` to the path of the nightly
binary.

### How do I use this with [Lune](https://github.com/lune-org/lune)?
Follow the [Editor Setup guide](https://lune-org.github.io/docs/getting-started/4-editor-setup).
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
