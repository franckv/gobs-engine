# gobs-engine

[![Rust](https://github.com/franckv/gobs-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/franckv/gobs-engine/actions/workflows/rust.yml)

Game engine written in Rust based on Vulkan.

## Crates
* gobs: public API. The only crate you need to include in your project
* gobs-render: rendering engine, backend agnostic via HAL abstraction (currently only Vulkan implemented)
* gobs-resource: resource manager. Manages async loading / unloading of resources (mesh, textures, materials, ...)
* gobs-scene: scene graph. Represent your world data to be rendered
* gobs-game: main loop and input management. Your application should implement the GobsGame trait
* gobs-egui: egui integration for in-game / debugging UI

## Examples

See /examples folder
