# Scene renderer

This project provides a tiny, deterministic scene renderer used as the base
repository for the course. It models scenes composed of rectangles and renders
them to a line-based string output.

This repository is intentionally minimal and will be extended throughout the
semester.

## Build

```bash
cargo build
```

## Test

```bash
cargo test
```

## CI Expectations

All assignments require the GitHub Actions CI checks to be passing. The CI
workflow runs formatting, linting, and the test suite on every push and pull
request.
