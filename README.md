# zed-luau
A [Zed](https://zed.dev/) extension that adds support for the [Luau programming language](https://luau-lang.org/).

### Installation
To install zed-luau, you can use the extension menu in Zed, or clone the
repository and install it as a dev extension with `zed: install dev extension`.

### Configuring
This extension can be configured using lsp settings. The default configuration
looks like this:
```json
{
	// ...
	"lsp": {
		// ...
		"luau-lsp": {
			"settings": {
				"luau-lsp": {},
				"ext": {
					"roblox": {
						/// Whether or not Roblox-specific features should be enabled.
						"enabled": false,
						/// The security level of scripts.
						/// Must be "roblox_script", "local_user", "plugin" or "none".
						"security_level": "roblox_script"
					},
					"fflags": {
						/// Whether or not all boolean, non-experimental fflags
						/// should be enabled by default.
						"enable_by_default": true,
						/// Whether or not currently enabled FFlags should be synced
						/// with Roblox's currently published FFlags (only the ones
						/// starting with FFlagLuau).
						"sync": false,
						/// Flags that are forced to some value.
						"override": {}
					},
					/// Definition files to pass to the language server.
					/// If an element in this array begins with '/', it is interpreted as an
					/// absolute path, and otherwise as a relative path to the workspace
					/// root.
					"definitions": [],
					/// Documentation files to pass to the language server.
					/// If an element in this array begins with '/', it is interpreted as an
					/// absolute path, and otherwise as a relative path to the workspace
					/// root.
					"documentation": [],
					/// Whether or not the worktree binary, if any, should be preferred over
					/// installing the language server binary automatically and using that.
					/// It is important to set this to true if you're installing luau-lsp with
					/// Aftman or Foreman, for example.
					"prefer_worktree_binary": false
				}
			}
		},
		// ...
	},
	// ...
}
```

Note that the `ext` settings are read only by this extension, while the `luau-lsp` settings are read directly by the language server itself. The
configuration options for the latter can be viewed here:
https://github.com/JohnnyMorganz/luau-lsp/blob/ae63ce5e10bc5d42122669fc20606fc5ec2fe54d/src/include/LSP/ClientConfiguration.hpp#L220.

As an example, if you want to enable inlay hints and use strict datamodel
types, your settings may look like this:
```json
{
	// ...
	"inlay_hints": {
		"enabled": true
	},
	// ...
	"lsp": {
		// ...
		"luau-lsp": {
			"settings": {
				"luau-lsp": {
					"inlayHints": {
						"parameterNames": "all"
					},
					"diagnostics": {
						"strictDatamodelTypes": true
					}
				}
			}
		},
		// ...
	},
	// ...
}
```

### Additional information for Roblox users
If you're a Roblox developer, you probably want to enable the `roblox` setting.
If you're using Rojo, you should also add something along the lines of the
following to your [Zed tasks](https://zed.dev/docs/tasks):
```json
{
	{
		"label": "Rojo autogenerate sourcemap",
		"command": "rojo sourcemap --watch --output sourcemap.json --include-non-scripts",
		"use_new_terminal": true
	},
	{
		"label": "Rojo serve default",
		"command": "rojo serve default.project.json",
		"use_new_terminal": true
	},
	{
		"label": "Write assets from out.rbxl with lune",
		"command": "lune run write_assets"
	},
	{
		"label": "Rojo build out.rbxl",
		"command": "rojo build --output out.rbxl"
	}
}
```

### Having issues?
You can start by checking the logs using `zed: open log`. If zed-luau emitted
an error, you will find it there. You can also open the language server logs
with `zed: open language server logs`.

### More tools
For automatically formatting your code, see
https://github.com/JohnnyMorganz/StyLua. If you install StyLua you can use it
in Zed by setting it as the formatter for the languages you want. Your `settings.json` may then look like this:
```json
{
	// ...
	"languages": {
		// ...
		"Luau": {
			// ...
			"formatter": {
				"external": {
					"command": "stylua",
					"arguments": ["-"]
				}
			},
			// ...
		},
		// ...
	},
	// ...
}
```
