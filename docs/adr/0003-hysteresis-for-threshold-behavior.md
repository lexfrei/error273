# 0003. Hysteresis for every threshold-driven behavior

- Status: accepted
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

A single threshold made citizens oscillate at the edge of the warm zone, flipping behavior every tick as a value hovered around the boundary.

## Decision

Every threshold-driven behavior uses two thresholds, one to enter and one to leave. A threshold is either a constant or a pure function of the citizen's position: warmth's acting mark is the cost of getting home plus a margin, so it rises as a citizen walks away from the fire. What this decision guarantees is the gap between the two marks rather than either value, because the gap is what stops the oscillation. Warmth-seeking, rest, and colony-wide wood diversion to construction all follow this shape today, and new threshold-driven behaviors are expected to follow the same pattern.

## Consequences

Citizens no longer oscillate at zone edges, wherever they are standing. The band width is an ordinary tunable and tests pin it directly, so a threshold that starts moving with position cannot quietly narrow the band it is carried on.
