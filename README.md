# Metaverse Client
[![last-commit][last-commit-badge]][last-commit] [![open-pr][open-pr-badge]][open-pr] [![open-issues][open-issues-badge]][open-issues]

https://github.com/user-attachments/assets/80d14817-2224-4adc-a4c3-ec30eee408ed

## Rust libraries for creating metaverse clients
Metaverse Client is a package that can be used for easily creating fast, multithreaded metaverse clients. The goal of the project is to be as modular as possible, to allow for easy and straightforward testing, and enable users to quickly get started and implement changes. 

## Crates 
### Session 
[![crates.io-session][crates.io-session-badge]][crates.io-session] [![docs.rs-session][docs.rs-badge]][docs.rs-session]

The session crate is the core of the project. This is what handles packet IO, and server/ui communication using [actix](https://github.com/actix/actix). UnixDatagrams are used to communicate between the core and the UI. This allows frontends to be completely decoupled from the session, and can be written in any language using any framework. 
The server accepts messages in the form of packets as defined by the [spec](https://wiki.secondlife.com/wiki/Category:Messages), and returns UI events serialized into bytes. 
### Messages 
 [![crates.io-messages][crates.io-messages-badge]][crates.io-messages] [![docs.rs-messages][docs.rs-badge]][docs.rs-messages]

This is the protocol spec. This contains all of the information about messages that can be sent to and from the server. The goal is to keep this general enough to be able to implement this for both client and server projects. More information on the spec can be found [here](https://wiki.secondlife.com/wiki/Category:Messages). 

### Environment
[![crates.io-environment][crates.io-environment-badge]][crates.io-environment] [![docs.rs-environment][docs.rs-badge]][docs.rs-environment]

Provides a straightforward way of handling incoming Layer Data information being sent from an open metaverse server. This is intended to handle all of the hard parts of working with 3d data, allowing frontend applications to simply access the generated 3d files and render them where they need to go. 

### Ui
This is a debug UI. Written in bevy and bevy-egui, it is not expected to get very polished. The more user-friendly UI can be found at [benthic_viewer](https://github.com/benthic-mmo/benthic_viewer). 

## Getting Started 
``cargo run``
Will run the debug UI.
``cargo test`` 
Will run the tests. 



[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square

[crates.io-environment-badge]: https://img.shields.io/crates/v/metaverse_environment?logo=rust&logoColor=white&style=flat-square
[crates.io-environment]: https://crates.io/crates/metaverse_environment
[docs.rs-environment]: https://docs.rs/metaverse_environment/latest/metaverse_environment/

[last-commit-badge]:https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/

[open-pr-badge]:https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls

[open-issues-badge]:https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square

[crates.io-session-badge]: https://img.shields.io/crates/v/metaverse_session?logo=rust&logoColor=white&style=flat-square
[crates.io-session]: https://crates.io/crates/metaverse_messages
[docs.rs-session]: https://docs.rs/metaverse_session/latest/metaverse_session/

[crates.io-messages-badge]: https://img.shields.io/crates/v/metaverse_messages?logo=rust&logoColor=white&style=flat-square
[crates.io-messages]: https://crates.io/crates/metaverse_messages
[docs.rs-messages]: https://docs.rs/metaverse_messages/latest/metaverse_session/

[last-commit-badge]:https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/

[open-pr-badge]:https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls

[open-issues-badge]:https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
