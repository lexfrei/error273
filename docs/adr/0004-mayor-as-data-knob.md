# 0004. The mayor is a data knob, not a controller

- Status: accepted
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

The mayor is the entry point for external influence in a zero-player game, and later layers such as legitimacy and the acceptability of policies need to be able to scale that influence over time.

## Decision

The mayor is a plain resource holding per-building-type bias weights that are added to citizens' vote tally, defaulting to neutral and carrying no logic of its own, and legitimacy scales the magnitude of that added weight. Directives and standing policies travel on a separate channel where legitimacy is instead the probability that the mayor is obeyed at all, so influence over a vote and influence over an order are two different mechanisms. A directive that is not followed emits a chronicle line naming why.

## Consequences

Citizens' votes remain the primary signal driving outcomes. Later systems can multiply or gate the mayor's weight to model legitimacy and policy effects without touching the voting mechanism itself, and the probabilistic channel keeps the mayor from ever being omnipotent even at full standing.
