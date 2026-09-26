# Benthic Default Asset Converter

[![last-commit][last-commit-badge]][last-commit] [![open-pr][open-pr-badge]][open-pr] [![open-issues][open-issues-badge]][open-issues]

# Default Asset Converter 
The purpose of this crate is to allow assets from the benthic default asset repo to be importable into rust projects. This crate is entirely a build script and a lib.rs. 
This project downloads the default assets from a tagged version on GitHub of the [Benthic Default Assets repo](https://github.com/benthic-mmo/benthic_default_assets), and converts them to serde JSON represenations of rust structs. 
This allows the default assets to live in a user-viewable and language agnostic format, while still maintaining the benefits of rust-native objects.  

## Features

The togglable feature flag "animations" can be set to generate the static animations.

## Tests

When adding a new animation to the default asset repo, tests can be run to verify the animation.
Run using
`cargo test --features animations`

[![crates.io-core][crates.io-core-badge]][crates.io-core] [![docs.rs-core][docs.rs-badge]][docs.rs-core]

[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square
[crates.io-core-badge]: https://img.shields.io/crates/v/benthic_default_asset_converter?logo=rust&logoColor=white&style=flat-square
[crates.io-core]: https://crates.io/crates/benthic_default_asset_converter
[docs.rs-core]: https://docs.rs/crate/benthic_default_asset_converter/latest
[last-commit-badge]: https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/
[open-pr-badge]: https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls
[open-issues-badge]: https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
