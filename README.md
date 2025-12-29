# Metaverse Client

[![last-commit][last-commit-badge]][last-commit] [![open-pr][open-pr-badge]][open-pr] [![open-issues][open-issues-badge]][open-issues]

https://github.com/user-attachments/assets/80d14817-2224-4adc-a4c3-ec30eee408ed

## Rust libraries for creating open metaverse clients

This package is meant to be used as a platform for creating fast, multithreaded open metaverse clients.

## Goals

### Documented

This project aims to be fully documented and developer friendly. High level protocol docs can be found at [benthic-mmo.github.io](https://benthic-mmo.github.io), and lower level packet parsing and implementation docs can be found on [docs.rs](docs.rs). If you find that docs are missing or confusing feel free to open an issue here, or on the [docs repo](https://github.com/benthic-mmo/benthic-mmo.github.io).

### Modular

Each part of this project is meant to be useful on its own to other projects. Wherever possible, separate layers of the project are broken out into publicly facing interafaces, using accessible, language agnostic protocols like JSON or UDP.

### Community

People write code for other people. The community surrounding this project aims to be deliberate, welcoming and curious. Getting started should be simple, and implementing changes should be hassle-free.

### Standardized

Wherever possible, this project prioritizes well-documented and standardized tools over handwritten ones. Ideally this can serve as an interface between the more confusing secondlife/opensim protocols, and newer, cleaner ones.

### Testable

This project should prioritize writing machine-testable code to ensure stability long-term.

## Crates

### Core

[![crates.io-core][crates.io-core-badge]][crates.io-core] [![docs.rs-core][docs.rs-badge]][docs.rs-core]

This crate is the core of the project. This is what handles packet IO, and server/ui communication using [actix](https://github.com/actix/actix). UDP packets are used to communicate between the core and the UI. This allows frontends to be completely decoupled from the core, and can be written in any language using any framework.
The server accepts messages in the form of packets as defined by the [spec](https://wiki.secondlife.com/wiki/Category:Messages), and returns UI events serialized into bytes.

### Messages

[![crates.io-messages][crates.io-messages-badge]][crates.io-messages] [![docs.rs-messages][docs.rs-badge]][docs.rs-messages]

This is the protocol spec. This contains all of the information about messages that can be sent to and from the server, both in UDP and HTTP. The goal is to keep this general enough to be able to implement this for both client and server projects. More information on the spec can be found [here](https://wiki.secondlife.com/wiki/Category:Messages).

### Environment

[![crates.io-environment][crates.io-environment-badge]][crates.io-environment] [![docs.rs-environment][docs.rs-badge]][docs.rs-environment]

Provides a straightforward way of handling incoming terrain information being sent from an open metaverse server.

### Inventory

[![crates.io-inventory][crates.io-inventory-badge]][crates.io-inventory] [![docs.rs-inventory][docs.rs-badge]][docs.rs-inventory]
Provides handling for the on-disk inventory database. Stored as SQL, this allows for lookup of all received objects, and handles storage and retrieval of the user's inventory.

### Agent

[![crates.io-agent][crates.io-agent-badge]][crates.io-inventory] [![docs.rs-agent][docs.rs-badge]][docs.rs-agent]
Contains information on how to build skinned agents. This includes information like the agent struct, and functions for building agent skeletons.

### Ui

[![crates.io-agent][crates.io-agent-badge]][crates.io-inventory] [![docs.rs-agent][docs.rs-badge]][docs.rs-agent]
Both a debug UI and a bevy plugin. This allows for quick testing, and other bevy projects to implement the full viewer protocol using only a few lines of code.

## Getting Started

The benthic project stretches across several repos, containing several different crates. There are four primary crates that contain the project's core code.

- [Metaverse Client (this repo)](https://github.com/benthic-mmo/metaverse_client)
- [Metaverse Mesh](https://github.com/benthic-mmo/metaverse_mesh), used by the project to generate mesh data for displaying in the UI
- [Serde LLSD](https://github.com/benthic-mmo/serde-llsd), parser functions for open metaverse serialization formats.
- [Benthic Viewer (optional)](https://github.com/benthic-mmo/benthic_viewer), a godot frontend for the project. Optional for most development.

In order to test the project from the master branch, these dependencies must all be downloaded on your disk, and organized like this. On the master branch, these crates use relative paths for versioning, allowing changes in one crate to be built from the core.

```
benthic_project
  ├── metaverse_client/
  ├── metaverse_gltf/
  ├── benthic_viewer/
  └── serde-llsd/

OpenSimulator
  └── bin/
```

In order to test locally, an instance of OpenSimulator must also be running either on-disk or remotely.

Prerequisite Packages:

- `rust`
- `cargo`
- `openssl-devel`
- `alsa-lib-devel`
- `rust-libudev-devel`
- `rust-libdbus-devel`

### Building

`cargo run`
Will run the debug UI.
`cargo test`
Will run the tests.

[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square
[crates.io-environment-badge]: https://img.shields.io/crates/v/metaverse_environment?logo=rust&logoColor=white&style=flat-square
[crates.io-environment]: https://crates.io/crates/metaverse_environment
[docs.rs-environment]: https://docs.rs/metaverse_environment/latest/metaverse_environment/
[last-commit-badge]: https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/
[open-pr-badge]: https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls
[open-issues-badge]: https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
[docs.rs-badge]: https://img.shields.io/badge/docs-Docs.rs-red?&style=flat-square
[crates.io-core-badge]: https://img.shields.io/crates/v/metaverse_core?logo=rust&logoColor=white&style=flat-square
[crates.io-core]: https://crates.io/crates/metaverse_core
[docs.rs-core]: https://docs.rs/metaverse_session/latest/metaverse_core/
[crates.io-messages-badge]: https://img.shields.io/crates/v/metaverse_messages?logo=rust&logoColor=white&style=flat-square
[crates.io-messages]: https://crates.io/crates/metaverse_messages
[docs.rs-messages]: https://docs.rs/metaverse_messages/latest/metaverse_session/
[last-commit-badge]: https://img.shields.io/github/last-commit/benthic-mmo/metaverse_client?logo=github&style=flat-square
[crates.io-inventory-badge]: https://img.shields.io/crates/v/metaverse_inventory?logo=rust&logoColor=white&style=flat-square
[crates.io-inventory]: https://crates.io/crates/metaverse_inventory
[docs.rs-inventory]: https://docs.rs/metaverse_session/latest/metaverse_inventory/
[crates.io-agent-badge]: https://img.shields.io/crates/v/metaverse_agent?logo=rust&logoColor=white&style=flat-square
[crates.io-agent]: https://crates.io/crates/metaverse_agent
[docs.rs-agent]: https://docs.rs/metaverse_session/latest/metaverse_agent/
[crates.io-ui-badge]: https://img.shields.io/crates/v/metaverse_ui?logo=rust&logoColor=white&style=flat-square
[crates.io-ui]: https://crates.io/crates/metaverse_ui
[docs.rs-ui]: https://docs.rs/metaverse_session/latest/metaverse_ui/
[last-commit]: https://github.com/benthic-mmo/metaverse_client/commits/main/
[open-pr-badge]: https://img.shields.io/github/issues-pr/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-pr]: https://github.com/benthic-mmo/metaverse_client/pulls
[open-issues-badge]: https://img.shields.io/github/issues-raw/benthic-mmo/metaverse_client?logo=github&style=flat-square
[open-issues]: https://github.com/benthic-mmo/metaverse_client/issues
