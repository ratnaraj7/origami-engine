# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.5](https://github.com/ratnaraj7/origami-engine/compare/origami-macros-v0.1.0-alpha.4...origami-macros-v0.1.0-alpha.5) - 2024-10-06

### Added

- [**breaking**] Use `Ident` for component calls and update `anon` to return `concat_args`

## [0.1.0-alpha.4](https://github.com/ratnaraj7/origami-engine/compare/origami-macros-v0.1.0-alpha.3...origami-macros-v0.1.0-alpha.4) - 2024-10-02

### Other

- pass concat args to comp call
- improve logic/optimize
- use full path for `Span` in `combine_to_lit` macro
- remove unnecessary quote
- add `html_escape` and `minify_html` crate

## [0.1.0-alpha.3](https://github.com/ratnaraj7/origami-engine/compare/origami-macros-v0.1.0-alpha.2...origami-macros-v0.1.0-alpha.3) - 2024-09-30

### Other

- allow script/style content to be empty

## [0.1.0-alpha.2](https://github.com/ratnaraj7/origami-engine/compare/origami-macros-v0.1.0-alpha.1...origami-macros-v0.1.0-alpha.2) - 2024-09-29

### Added

- add `minify_html` feature, and make script moveable
- add `nominify` and `script_use` keywords
- add support for match expression
- generate macros for components instead of structs

### Fixed

- [**breaking**] add semicolon after `script_use`
- [**breaking**] remove unnecessary brace from `macro_rules`
- change order of `!`
- use semicolon to parse expr correctly
- [**breaking**] change feature name html-escape to html_escape
- repository url

### Other

- add license
- import escape,noescape only when `html_escape` is enabled
- remove unnecessary include children
- add todo comment
- use `IndexMap` for insertion order, optimize string concatenation, and add tests
- internal macro to combine strings to lit
- separate folder for children mod
- separate attribute module
- fix comment
- Merge pull request [#4](https://github.com/ratnaraj7/origami-engine/pull/4) from ratnaraj7/dev
- remove unnecessary imports

## [0.1.0-alpha.1](https://github.com/ratnaraj7/origami-engine/releases/tag/origami-macros-v0.1.0-alpha.1) - 2024-08-19

### Other
- add workflows
- initial commit
- add projects
