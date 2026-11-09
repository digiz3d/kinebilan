# KinéBilan

<p align="center">
  <img src="assets/icon.png" alt="KinéBilan icon" width="128">
</p>

A small desktop app for managing physiotherapy patients and their treatment sessions.

## Features

- Create, edit and delete patient records
- Track sessions per patient (date, objectives, content, notes)
- Search patients

## Build & run

Requires Rust (see `mise.toml`) and CMake. SDL3 is built from source.

```sh
cargo run --release
```

To run the GPUI-rendered application (in parallel with the SDL version):

```sh
cargo run --bin gpui_preview
```

To run the GPUI Kit version with Kit buttons, text inputs, textareas and scrollbars:

```sh
cargo run --release --features gpui-kit --bin gpui_kit_preview
```

All three versions share the same encrypted patient file. Run only one at a time to avoid overwriting changes made in another version.

Run the Kit UI and shared model/storage tests with:

```sh
cargo test --features gpui-kit-tests --bin gpui_kit_preview
```

## Data

Patient data is stored encrypted in `~/.kinebilan/patients.json` (or `./patients.json` if `HOME` is not set) and protected by the password you create on first launch.

## License

See the repository for license details.
