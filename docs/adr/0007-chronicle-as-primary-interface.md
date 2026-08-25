# 0007. A chronicle is the primary interface

- Status: proposed
- Date: 2026-08-25

## Context

Culture, traditions, votes, and deaths are all invisible on an ASCII map that only shows citizen positions and terrain.

## Decision

An event log under the map records notable events with names and dates. Simulation time compresses when nothing noteworthy is happening and slows down around events.

## Consequences

What counts as legible drives what counts as an event, which in turn makes naming individual citizens necessary.
