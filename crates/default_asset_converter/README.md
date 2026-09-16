# Benthic Asset Pipeline

The purpose of this repo is to allow assets from the benthic default asset repo to be importable into rust projects. This repo is entirely a build script, which converts the assets in the default assets repo into static importable rust files, and a lib.rs which includes the generated files in a usable library.

## Features

The togglable feature flag "animations" can be set to generate the static animations.

## Tests

When adding a new animation to the default asset repo, tests can be run to verify the animation.
Run using
`cargo test --features animations`
