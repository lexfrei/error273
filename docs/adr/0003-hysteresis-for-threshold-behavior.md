# 0003. Hysteresis for every threshold-driven behavior

- Status: accepted
- Date: 2026-08-25

## Context

A single threshold made citizens oscillate at the edge of the warm zone, flipping behavior every tick as a value hovered around the boundary.

## Decision

Every threshold-driven behavior uses two thresholds, one to enter and one to leave: warmth-seeking, rest, and colony-wide wood diversion to construction all follow this shape today, and new threshold-driven behaviors are expected to follow the same pattern.

## Consequences

Citizens no longer oscillate at zone edges. The enter and leave values are ordinary tunables, and tests pin the exact band edges so regressions are caught immediately.
