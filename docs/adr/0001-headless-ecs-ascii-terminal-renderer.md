# 0001. Headless ECS with an ASCII terminal renderer

- Status: accepted
- Date: 2026-08-25

## Context

This is a zero-player game: the simulation is the product, and rendering only needs to make it legible to a watching human, not to drive interaction. Candidates considered were Go with Ebitengine, TypeScript with PixiJS, and Godot.

## Decision

Rust with Bevy, running the `default_app` feature set with no window and no GPU, drawing the map as ASCII through ANSI escape codes to the terminal.

## Consequences

The prototype builds in seconds and the simulation logic is fully testable headless, with the renderer as one replaceable system among many. There is no editor and no GPU-accelerated UI, so all rendering and layout code is hand-rolled.
