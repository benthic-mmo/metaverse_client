# Metaverse Core

[![last-commit][last-commit-badge]][last-commit] [![open-pr][open-pr-badge]][open-pr] [![open-issues][open-issues-badge]][open-issues]

## Core

[![crates.io-core][crates.io-core-badge]][crates.io-core] [![docs.rs-core][docs.rs-badge]][docs.rs-core]
Provides an easy, asyncronous way of interacting with open metaverse servers in Rust. Handles packet IO between client and server, exposing a UDP interface for simple integration into other projects. This is meant to operate as an easily-bootstrappable backend for new open metaverse viewer projects.

Messages are handled using [actix](https://github.com/actix/actix), and accepts messages in the form of packets as defined by the [spec](https://wiki.secondlife.com/wiki/Category:Messages), and returns UI events serialized into bytes.

## Getting Started

`cargo run`
Will run the debug UI
`cargo test -- --nocapture`
Will run tests with debug mode on.

[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square
[crates.io-core-badge]: https://img.shields.io/crates/v/metaverse_core?logo=rust&logoColor=white&style=flat-square
[crates.io-core]: https://crates.io/crates/metaverse_core
[docs.rs-core]: https://docs.rs/metaverse_core/latest/metaverse_core/
[last-commit-badge]: https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/
[open-pr-badge]: https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls
[open-issues-badge]: https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
