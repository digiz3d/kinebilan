# Save format changes

Save code (not save data) lives in `src/storage/`. The current version is `CURRENT_VERSION` in `src/storage/mod.rs`.

To add version N+1:

1. Freeze the current shapes you're changing: copy the old structs into `src/storage/migrations/vN_to_vN+1.rs` (e.g. `PatientV1`). Never edit frozen structs later.
2. Write `migrate(raw: Value) -> Result<Value, serde_json::Error>`: deserialize into the frozen structs, convert to the new shapes, and return the file at version N+1 (`{ "version": N+1, "patients": [...] }`).
3. Register it in `migrations::step` in `src/storage/migrations/mod.rs`.
4. Bump `CURRENT_VERSION` in `src/storage/mod.rs`.
5. Update the live structs in `src/patient.rs` to the new shape.
6. Add tests in the migration file: old JSON in, expected new JSON out. Add a test that old fixtures still load.
7. Run `cargo test`.
