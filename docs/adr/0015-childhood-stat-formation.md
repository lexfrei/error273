# 0015. Stats are formed in childhood, not rolled at birth

- Status: proposed
- Date: 2026-08-25

## Context

Rolling strength, wits and hardiness on a species range at birth makes a citizen's whole history irrelevant to who they are, and the colony already carries two transmission systems — the household and the tradition card — that a third would only duplicate.

## Decision

A citizen's stats are the output of a multiplicative accumulator running from birth to `ADULT_AGE` over the warmth and food the colony actually had, weighted heaviest at the start, and resolving at three or four named milestone ages rather than continuously or once. Hardiness and strength keep a diminishing adolescent catch-up term so a half-starved cohort can be partly repaired, while wits closes hard at adulthood; hardiness additionally carries a slow acclimatisation term fed by hours spent below freezing and decaying when they stop. Transmission uses the two channels that exist and adds none: the household is the vertical channel and already hands down a trade, tradition cards spreading by contact are the horizontal one, and the oblique channel — a school, the trades standing around a child — is a card contact with an adult from outside the child's own household. The existing lifespan roll survives as the unexplained residual on top of what the colony raised, and the share of the outcome it carries scales with colony prosperity, so a comfortable colony raises citizens who differ for reasons it cannot see.

## Consequences

Two children of one household are not clones and the reason they differ is cited rather than asserted, while a colony's decade of plenty or famine becomes readable a generation later in the people it produced. The accumulator costs one field per child and no new scan, because the colony-wide warmth and food shares it integrates are already computed every tick for the maturation rate; the milestones cost extra state and extra chronicle variants, accepted so that a child visibly becomes someone during a run.
