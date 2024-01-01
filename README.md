# Recommend me

An [`egui`](https://github.com/emilk/egui) app for recommending movies.

## TODO

See [MVP milestone](https://github.com/m-spitfire/meepls/milestone/1).

## Running

```
cargo run --release
```

## Development

Crates:
- `./src` (to be changed?): Entry point of egui app
- `./lib` (to be changed?): Hot module of egui app (basically everything)

### Requirements
- rust
- [`cargo-watch`](https://crates.io/crates/cargo-watch) (if you want hot-reloading)

To run with hot-reloading enabled

```
cargo watch -w lib -x "build -p lib" &
cargo run --features reload
```

Or just normal mode

```
cargo run
```
