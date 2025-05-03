# Metaverse Environment 

[![last-commit][last-commit-badge]][last-commit] [![open-pr][open-pr-badge]][open-pr] [![open-issues][open-issues-badge]][open-issues]

This crate is for handling the 3d environment of opensimulator worlds. This will contain the logic for handling LayerData, objects, and other components related to the 3d world. Ideally this will become a shared library similar to metaverse_messages that can be used both client and server side for handling 3d environments.

## Environment
[![crates.io-environment][crates.io-environment-badge]][crates.io-environment] [![docs.rs-environment][docs.rs-badge]][docs.rs-environment]

Provides a straightforward way of handling incoming Layer Data information being sent from an open metaverse server. This is intended to handle all of the hard parts of working with 3d data, allowing frontend applications to simply access the generated 3d files and render them where they need to go. 

## Getting Started 
``cargo run``
Will run the debug UI
``cargo test -- --nocapture``
Will run tests with debug mode on.



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
