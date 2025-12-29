# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added support for explicit type parameter instantiation.
- Added support for comment highlights via the zed-comment extension.

# Changed

- Changed endpoint for all requests except luau-lsp download, making GitHub rate limits less likely
  and easier to work around (just use a local binary).

## [0.3.3] - 2025-10-16

### Fixed

- Fixed error when having multiple windows on Windows

## [0.3.2] - 2025-10-1

### Changed

- Bumped extension API to latest version.
- Removed now-unnecessary Windows `current_dir` workaround.

## [0.3.1] - 2025-8-2

### Added

- Added support for `declare extern type with` syntax

### Fixed

- Fixed binary auto-install failing on X86-64 linux.
- Fixed line comment being automatically inserted after pressing enter at the end of a line that
  starts a block comment.

## [0.3.0] - 2025-7-4

### Added

- Added support for the companion plugin.
- Added support for putting extension settings directly under `lsp.luau-lsp.settings`.
  * This is now recommended but *if zed-luau finds `ext` in settings (project settings or regular
    settings, project settings being preferred), it will prefer that and ignore other settings.*
- Added support for text objects ([docs](https://zed.dev/docs/vim#treesitter)).
- Added settings for more fine-grained control over Roblox-related behavior.
  * roblox.download_api_documentation
  * roblox.download_definitions

### Fixed

- Fixed default security level not being plugin.
- Fixed being unable to depend on automatically added Roblox types in additional definition files.
- Fixed doc comments not being inserted with `extend_comment_on_newline`.

## [0.2.2] - 2024-12-22

### Added

- Added support for declaration syntax.
- Added setting ext.fflags.enable_new_solver.

### Changed

- Updated to the latest grammar commit.
- Made syntax highlighting captures more specific.
- Constrained special highlighting for `string` to only when it's the table in a dot index
  expression.

## [0.2.1] - 2024-11-24

### Added

- Added outline support with annotations and special handling for table types.

### Changed

- Changed the grammar for Luau to [4teapo/tree-sitter-luau](https://github.com/4teapo/tree-sitter-luau)
  for improved syntax highlighting.
- Changed default script security level to plugin security.

### Removed

- Removed automatic closing for `<`/`>`.

### Fixed

- Fixed bracket pairs inside of comments automatically closing.

## [0.2.0] - 2024-10-26

### Added

- Added `ext.binary` settings, including `ext.binary.path` and `ext.binary.ignore_system_version`.
- The following pairs of keywords now get highlighted as brackets:
  - `then`/`end`
  - `do`/`end`
  - `function`/`end`
  - `repeat`/`until`
  - `"`/`"`
  - \`/\`
  - `<`/`>`
  - `[[`/`]]`
- Added support for FInt, DFFlag, and DFInt FFlags.
- Added automatic indentation support.
- Added support for automatically closing ', ` and <. They're currently inserted automatically in
  comments as well due to a bug in Zed.
- Added runnable for Jest Lua tests (`it("", ...)`, `describe("", ...)`, and `test("", ...)`) with
  the tag `luau-jest-test`.

### Changed

- Changed error messages for configuration faults.
- Changed the default value of `ext.fflags.enable_by_default` from `true` to `false`.
- Changed the default value of `ext.fflags.sync` from `false` to `true`.

### Removed

- Removed the `ext.prefer_worktree_binary` setting in favor of `ext.binary.ignore_system_version`.
