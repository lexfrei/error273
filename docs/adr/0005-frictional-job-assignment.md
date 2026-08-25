# 0005. Job assignment produces mismatches for mechanical reasons

- Status: proposed
- Date: 2026-08-25

## Context

Debuffs for citizens working the wrong job are meaningless if job assignment is an omniscient optimizer that never produces a mismatch in the first place.

## Decision

Assignment is local and frictional: the nearest idle citizen takes an open vacancy, nobody quits a job on their own, children inherit a parent's trade, scarcity forces bad fits, and urgent needs override aptitude. Per-citizen stats are hidden and are only revealed through noisy performance over time. A mismatch costs output, raises fatigue, and increases accidents and discontent.

## Consequences

Emergent "wrong person, wrong job" stories happen without any randomness in the mechanism. A future policy that reassigns citizens by aptitude becomes a meaningful, costly lever rather than a no-op.
