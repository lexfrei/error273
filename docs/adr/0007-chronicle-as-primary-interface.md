# 0007. A chronicle is the primary interface

- Status: proposed
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

Culture, traditions, votes, and deaths are all invisible on an ASCII map that only shows citizen positions and terrain.

## Decision

Notable events are emitted by explicit hooks over a closed event set, each entry carrying a name and a date. The sparse permanent record, browsable by year, is the primary carrier of comprehension, while the log under the map is a peripheral rolling ticker the watcher can miss without losing the thread, because at this tempo a feed scrolls faster than anyone reads. Simulation time compresses when nothing noteworthy is happening and slows down around events, reading the same event table that decides what is worth recording.

## Consequences

What counts as legible drives what counts as an event, which in turn makes naming individual citizens necessary. One table serves both the chronicle and the tempo controller, so a change to what is worth reading is automatically a change to what is worth slowing for.
