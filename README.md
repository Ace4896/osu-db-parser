# osu! Database Viewer

A small app for viewing the contents of osu!stable's database files. It is capable of viewing:

- `osu.db` - Information about installed beatmaps
- `collection.db` - Information about beatmap collections
- `scores.db` - Information about scores achieved locally

The formats for each database are described in more detail on the [osu! wiki](https://github.com/ppy/osu/wiki/Legacy-database-file-structure).

## Requirements

To run the app natively, the following needs to be setup:

- Stable Rust (tested on 1.71.0)
- [eframe Dependencies](https://github.com/emilk/eframe_template#testing-locally)

When building for WASM, a few extra things are also needed:

- Add the WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/)

# Development Usage

The app is split into two crates:

- [`parser`](./parser): A parsing library for the osu! database file formats
- [`viewer`](./viewer): The database viewing app

Use one of the following commands to run the app:

```bash
# Run the app natively
cargo run               # Debug
cargo run --release     # Release

# Build and serve the app for WASM
# The compiled output can be found in ./viewer/dist
# It can be viewed at http://127.0.0.1:8080
# NOTE: The output is cached; after rebuilding, either force refresh or use http://127.0.0.1:8080#dev
trunk serve ./viewer/index.html             # Debug
trunk serve --release ./viewer/index.html   # Release
```
