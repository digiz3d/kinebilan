# KinéBilan

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

## Data

Patients are stored in `~/.kinebilan/patients.json` (or `./patients.json` if `HOME` is not set).

## License

See the repository for license details.
