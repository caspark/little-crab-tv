# Crab TV

A rasterizing software renderer, roughly following the approach laid out by https://github.com/ssloy/tinyrenderer

## Prereqs

* Rust, obviously
* eframe dependencies: `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`
* `cargo-watch` for devloop: `cargo install cargo-watch`

## Developing

```
cargo watch -x run
```

That'll run the renderer in debug mode, but if you want to play with it then it's best to run it in release mode so that it doesn't run like molasses:

```
cargo run --release
```
