# zed-luau
A Zed extension that adds support for the Luau programming language, a flavor
of Lua made by Roblox.

### Usage
This extension can be configured using lsp settings in your `settings.json`
file, which you can open using the `zed: open settings` command. See
https://zed.dev/docs/configuring-zed for more information. Settings that are
passed directly to the language server (luau-lsp) reside in `settings.luau-lsp`
and settings that only this extension reads are put in `settings.ext`.

The default configuration looks like this:
```json
{
	// ...
	"lsp": {
		// ...
		"luau-lsp": {
			"settings": {
				"ext": {
					"roblox": {
						/// Whether or not Roblox-specific features should be enabled.
						"enabled": false,
						/// The security level of scripts.
						"security_level": "roblox_script"
					},
					"fflags": {
						/// Whether or not all boolean, non-experimental fflags should be enabled by default.
						"enable_by_default": true,
						/// Whether or not currently enabled FFlags should synced with Roblox's currently published FFlags (only the ones starting with FFlagLuau).
						"sync": false,
						/// Flags that are forced to some value.
						"override": {}
					},
					/// Definition files to pass to the language server. If an element in this array begins with '/', it is interpreted as an absolute path, and otherwise as a relative path to the workspace root.
					"definitions": [],
					/// Documentation files to pass to the language server. If an element in this array begins with '/', it is interpreted as an absolute path, and otherwise as a relative path to the workspace root.
					"documentation": [],
					/// Whether or not the worktree binary, if any, should be preferred over installing the language server binary automatically and using that. It is important to set this to true if you're installing luau-lsp with Aftman or Foreman, for example.
					"prefer_worktree_binary": false
				}
			}
		},
		// ...
	},
	// ...
}
```

If zed-luau isn't working as it should, start by inspecting the logs using
`zed: open log` and `debug: open language server logs`. If zed-luau found an
error in your configuration, the error message can be viewed in the
(non-language-server) log menu. File an issue on github if you believe what
you're experiencing is a problem with the extension.

If there was an error in your configuration, you may need to reload the
workspace after fixing it, in order for the language server to start working
again.

You can directly configure the underlying language server using
`lsp.luau-lsp.settings.luau-lsp`. The settings that can be configured can be
viewed here:
https://github.com/JohnnyMorganz/luau-lsp/blob/ae63ce5e10bc5d42122669fc20606fc5ec2fe54d/src/include/LSP/ClientConfiguration.hpp#L220.

As an example, your configuration may look like this if you want inlay hints for
function parameters:
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
If you're a Roblox developer, you may want to enable the `roblox` setting. You
will probably also prefer adding the following Zed tasks (using `zed: open
tasks`):
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
	// edit if needed
	{
		"label": "Write assets from out.rbxl with lune",
		"command": "lune run write_assets"
	},
	// edit if needed
	{
		"label": "Rojo build out.rbxl",
		"command": "rojo build --output out.rbxl"
	}
}
```
### More tools
For auto-formatting your code, see https://github.com/JohnnyMorganz/StyLua.
If you install StyLua, you can use it for both Lua and Luau files, by setting
the `formatter` for these languages to StyLua. Your `settings.json` may then
look like this:
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
		"Lua": {
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
