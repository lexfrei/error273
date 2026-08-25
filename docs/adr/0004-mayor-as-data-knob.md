# 0004. The mayor is a data knob, not a controller

- Status: accepted
- Date: 2026-08-25

## Context

The mayor is the entry point for external influence in a zero-player game, and later layers such as legitimacy and the acceptability of policies need to be able to scale that influence over time.

## Decision

The mayor is a plain resource holding per-building-type bias weights that are added to citizens' vote tally. Weights default to neutral, and the mayor resource carries no logic of its own.

## Consequences

Citizens' votes remain the primary signal driving outcomes. Later systems can multiply or gate the mayor's weight to model legitimacy and policy effects without touching the voting mechanism itself.
