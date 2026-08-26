# 0011. Entertainers are load-bearing

- Status: accepted
- Date: 2026-08-25
- Amended: 2026-08-26

## Context

The cautionary case in the genre ships a full performer occupation with generated songs, poems and dances and visiting troupes, and its own documentation concludes that performers serve little practical purpose because an assigned performer does nothing the simulation would not have done anyway. Generative depth creates no weight; only changing an outcome does.

## Decision

A performance computes a quality percentage from enumerated terms — audience size, venue warmth, entertainer skill, tradition match — and quality maps to an outcome tier that pays in mood, in a colony-wide work-speed buff lasting a fixed number of days, and in a push on a culture axis. The score entertainment advertises to a citizen is bounded and falls as the need fills, and the arbiter samples weighted-random over the top few options rather than taking the best, so the colony cannot converge on one venue. Repeating one kind of entertainment builds tolerance at two thirds of the gain with a boredom band, and the number of distinct kinds needed to hold mood scales with prosperity per capita. A venue is scored on the quality of what it provides, not on existing.

## Consequences

An hour spent entertaining is an hour not spent hauling wood, so the payout must be marginal in a crisis and dominant on a plateau, which is where this layer's tuning risk sits. Entertainers multiply cultural transmission and normalisation but never buy a tradition card outright.

## Amendment, 2026-08-26, on what was built

Built as decided: quality from enumerated terms, the outcome tiers on the RimWorld distribution, a colony-wide work-speed buff for a named number of days, tolerance capped at two thirds of the gain, and variety pressure that scales with prosperity per capita. Four things departed from the letter of this decision, and one thing it did not anticipate turned out to matter more than any of them.

**The advertisement model is not built and is not needed.** This decision borrowed The Sims' bounded advertisement and top-N arbiter to stop a colony converging on one venue and to stop a citizen using entertainment until they collapse. Neither failure is reachable here, because recreation has no self-serve object at all: there is nothing a citizen can use, only somebody else's hour to attend. A citizen cannot loop on a performance any more than they can loop on being born. There is also exactly one venue -- the warm ground, which is where the audience already is -- so convergence on it is the design rather than a failure of it. What replaced the arbiter is a call: the evening is announced ahead, and a citizen whose survival tier is quiet walks to it the way they walk to the fire, starting when the walk is exactly as long as the time left.

**The boredom band scales the payout instead of switching a state.** RimWorld's tolerance band flags a pawn as bored of a type above one mark and un-bored below another. Here the same two marks ramp how much of a night's gain being tired of it takes, from nothing to the cap. A flag would have needed somewhere to be read, and the only honest reader is the payout, so the band went straight there.

**Variety scales as a rate, not as a count.** The decision says the number of distinct kinds needed to hold mood scales with prosperity. With two kinds a required count has nowhere to move, so what scales instead is how fast a colony tires of what it has: at its target stores at the full rate, at half of them at half, with nothing at all not at all. It is the same pressure expressed where it can act, and it produces the intended result -- a colony that has everything runs out of things to put on and a colony scraping by does not.

**The culture axis is deferred, not dropped.** The tradition term is a named argument to the quality function that contributes exactly zero, so the shape the culture layer plugs into exists and is visible in the code rather than being a promise.

**What this decision did not anticipate: a need with no remedy leaks.** Recreation is the first need in the game that nothing a citizen does alone can raise, and three separate structural readers of the needs table counted it without asking whether it was that kind of need -- what a citizen is worth at the work, whether the colony may grow, and whether anybody can be spared to go looking. The third stopped expeditions outright and none of them announced itself as anything but a shorter arc. Every structural reader now asks whether a need is one the colony runs on before counting it, and the control for it is that with no entertainer in the colony the build runs identical to the one before this layer, tick for tick to the last death.

**The tuning risk this decision named resolved half.** "Marginal in a crisis" is built and holds: the trade is only ever appointed while both stores are at or above target, a colony with nothing tires of nothing, and the founding party -- the leanest the colony ever is -- carries no entertainer at all. "Dominant on a plateau" is not achieved and is not claimed. Measured against the same build with no entertainer in it, over twelve worlds, the trade is clearly positive on mood and roughly even on the arc, and it is never dominant.

**The layer's closing measurement, on the question the arc could not answer.** Twenty-four worlds, four arms, one binary, the threshold and the arms fixed before the run: does a colony reach forty-five people, half again its founding party, inside forty years. No entertainer, five of twenty-four. As shipped, six. Share halved and appointment gate raised to one and a half times target, six each and byte-identical to shipped on every world, because at the twenty to thirty-five adults these colonies hold the share rounds to one either way and the gate only ever decides a first appointment made when the founding stores are far above either bar. Gates strict enough to bite -- two, three, five and nine times target -- move it to five, five, six and six. **Nineteen of twenty-four colonies stall at thirty-nine people whether or not they staff an entertainer, and no lever on this trade changes that.** The entertainer's hour is not what stops a colony growing; the labour margin is, and this trade is a rounding error against it. The share and the appointment gate stay where they are, not because they are tuned but because there is nothing here for them to fix.
