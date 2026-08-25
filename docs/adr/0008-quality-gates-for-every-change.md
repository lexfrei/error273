# 0008. Quality gates for every change

- Status: accepted
- Date: 2026-08-25

## Context

A regression in a threshold or a rate constant is easy to miss on an ASCII map that only shows citizen positions and counters, and a stale test binary once reported a green run of 3 tests while the source held 33.

## Decision

Every change must leave `rustfmt` clean and `clippy` free of warnings, fixing rather than suppressing anything it flags. Simulation logic is written test-first. Tests run as `cargo clean --package error273 && cargo test`, since a stale test binary was once observed surviving an interrupted build. A timed headless run is performed for every change, and its final status is logged in `docs/balance.md`.

## Consequences

The gate is cheap, mechanical, and reproducible by anyone working on the project.
