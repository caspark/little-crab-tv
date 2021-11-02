# Crab TV

A rasterizing software renderer, written in the spirit of https://github.com/ssloy/tinyrenderer

## Prereqs

* Rust, obviously
* eframe dependencies: `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`
* `cargo-watch` for devloop: `cargo install cargo-watch`

## Developing

```
cargo watch -x run
```
