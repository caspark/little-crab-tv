# Crab TV

A rasterizing software .obj model renderer written from-scratch in Rust, including implementing primitives like drawing lines and triangles. 

It supports textures, normal maps, lighting using a phong shading model, shadows, screen space ambient occlusion, and glow maps:

![output](https://github.com/caspark/little-crab-tv/assets/931544/d677a01d-5dce-464f-8279-afadd9497803)

Here's a slower-paced video that also shows off some more of the earlier and intermediate steps, such as wireframe rendering, flat shading and shadow mapping:

https://github.com/caspark/little-crab-tv/assets/931544/e4937660-4051-462e-ad12-e83fc643d64a

It roughly follows the overall approach laid out by the C++-oriented [Tiny Renderer](https://github.com/ssloy/tinyrenderer).

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
