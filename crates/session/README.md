
# Metaverse Session
![last-commit-badge] ![open-pr-badge] ![open-issues-badge]

## Session 
[![crates.io-session][crates.io-session-badge]][crates.io-session] [![docs.rs-session][docs.rs-badge]][docs.rs-session]
Provides an easy, asyncronous way of interacting with open metaverse servers in Rust. Handles packet IO between client and server, exposing a UnixDatagram interface for simple integration into other projects. This is meant to operate as an easily-bootstrappable backend for new open metaverse viewer projects.

Messages are handled using [actix](https://github.com/actix/actix), and accepts messages in the form of packets as defined by the [spec](https://wiki.secondlife.com/wiki/Category:Messages), and returns UI events serialized into bytes.

## Getting Started 
``cargo run``
Will run the debug UI
``cargo test -- --nocapture``
Will run tests with debug mode on.


[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square

[crates.io-session-badge]: https://img.shields.io/crates/v/metaverse_session?logo=rust&logoColor=white&style=flat-square
[crates.io-session]: https://crates.io/crates/metaverse_messages
[docs.rs-session]: https://docs.rs/metaverse_session/latest/metaverse_session/

[crates.io-messages-badge]: https://img.shields.io/crates/v/metaverse_messages?logo=rust&logoColor=white&style=flat-square
[crates.io-messages]: https://crates.io/crates/metaverse_messages
[docs.rs-messages]: https://docs.rs/metaverse_messages/latest/metaverse_session/

[last-commit-badge]:https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr-badge]:https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues-badge]:https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square

