# 0011. Entertainers are load-bearing

- Status: proposed
- Date: 2026-08-25

## Context

The cautionary case in the genre ships a full performer occupation with generated songs, poems and dances and visiting troupes, and its own documentation concludes that performers serve little practical purpose because an assigned performer does nothing the simulation would not have done anyway. Generative depth creates no weight; only changing an outcome does.

## Decision

A performance computes a quality percentage from enumerated terms — audience size, venue warmth, entertainer skill, tradition match — and quality maps to an outcome tier that pays in mood, in a colony-wide work-speed buff lasting a fixed number of days, and in a push on a culture axis. The score entertainment advertises to a citizen is bounded and falls as the need fills, and the arbiter samples weighted-random over the top few options rather than taking the best, so the colony cannot converge on one venue. Repeating one kind of entertainment builds tolerance at two thirds of the gain with a boredom band, and the number of distinct kinds needed to hold mood scales with prosperity per capita. A venue is scored on the quality of what it provides, not on existing.

## Consequences

An hour spent entertaining is an hour not spent hauling wood, so the payout must be marginal in a crisis and dominant on a plateau, which is where this layer's tuning risk sits. Entertainers multiply cultural transmission and normalisation but never buy a tradition card outright.
