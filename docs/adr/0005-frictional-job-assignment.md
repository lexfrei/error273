# 0005. Job assignment produces mismatches for mechanical reasons

- Status: proposed
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

Debuffs for citizens working the wrong job are meaningless if job assignment is an omniscient optimizer that never produces a mismatch in the first place.

## Decision

A vacancy is filled by scoring candidates on distance, prior experience in that trade and the mayor's bias, then sampling from the top few rather than taking the best; the distance term is a deliberately cheap and wrong metric, and the radius is unbounded for haulers but bounded for trade workers, with an escape valve for jobs nobody has taken. Nobody quits a job on their own, a child takes the household trade by default, scarcity forces bad fits, and urgent needs override aptitude. Per-citizen stats are hidden: the citizen card prints one bucketed adjective per stat, and only after the colony has observed enough worker-days in a trade that exercises it, so the printed word is the colony's running estimate and starts wrong before it converges. A mismatch resolves into the single focus multiplier of ADR 0010, whose consequences are output, accident chance and construction quality.

## Consequences

Emergent "wrong person, wrong job" stories happen without any randomness in the mechanism. A future policy that reassigns citizens by aptitude becomes a meaningful, costly lever rather than a no-op, and the wrong distance metric must be narrated in the chronicle or it reads as a pathfinding bug.
