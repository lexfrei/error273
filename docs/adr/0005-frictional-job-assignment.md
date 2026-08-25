# 0005. Job assignment produces mismatches for mechanical reasons

- Status: proposed
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

Debuffs for citizens working the wrong job are meaningless if job assignment is an omniscient optimizer that never produces a mismatch in the first place.

## Decision

A vacancy is filled by scoring candidates on distance, prior experience in that trade and the mayor's bias, then sampling from the top few rather than taking the best. Distance is the Chebyshev norm, which is the true travel time under one-cell-per-tick king moves and therefore serves movement, job and patch scoring and the return arithmetic alike, while the heat field keeps a Euclidean distance because that is physics rather than a decision; the resulting gap between a round warm ring and a square walk is accepted and narrated rather than reconciled. A hauler's radius is bounded by the realised-chunk index of ADR 0016 and by the marginal-value cutoff of ADR 0017 rather than being unlimited, trade workers keep a tighter radius, and an ageing job nobody has taken is the escape valve. Nobody quits a job on their own, a child takes the household trade by default, scarcity forces bad fits, and urgent needs override aptitude. Per-citizen stats are not rolled at birth; their origin is the childhood formation of ADR 0015. They stay hidden: the citizen card prints one bucketed adjective per stat, and only after the colony has observed enough worker-days in a trade that exercises it, with the warmth and food the citizen was raised in serving as a coarse band-level prior. The printed word is the colony's running estimate, starts wrong, and converges. A mismatch resolves into the single focus multiplier of ADR 0010, whose consequences are output, accident chance and construction quality.

## Consequences

Emergent "wrong person, wrong job" stories happen without any randomness in the mechanism. A future policy that reassigns citizens by aptitude becomes a meaningful, costly lever rather than a no-op, and the divergence between the round warm ring and the square walk must be narrated in the chronicle or it reads as a pathfinding bug.
