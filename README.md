# bhlool terminal snake game

This repository contains a single Rust crate that runs a colorful terminal Snake game.

## Right locations (canonical structure)

Only these locations are needed for the runnable game:

- `src/main.rs` — binary entrypoint (`game::run()`).
- `src/game.rs` — all game logic and rendering.
- `Cargo.toml` / `Cargo.lock` — crate manifest and lockfile.

There are no duplicate game folders in this repo.

## Run

```bash
cargo run
```

## Check

```bash
cargo fmt --check
cargo test
```
