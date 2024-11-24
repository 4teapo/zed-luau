# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
