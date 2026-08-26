# 0018. What a trip takes depends on the ground, and when to stop taking it

- Status: proposed
- Date: 2026-08-26

## Context

ADR 0016 gave the world richness that rises with distance and asserted that this, against a warmth cost that rises linearly, gives every trip an interior optimum. It does not. What a citizen carries is set by the citizen alone -- a load is the hauler's own capacity and nothing else enters it -- so a longer walk brings home no more in one go, and the optimum the record claims cannot exist. The consequence is the failure design.md §14 inherits from the genre: a colony with a regrowing near ring has no reason ever to leave it, and an unbounded world is decoration.

Two mechanisms from foraging theory close the gap, and design.md §14 already names both. Orians and Pearson's central place foraging says a forager carrying to a fixed base should bring more from further away, because a fixed round-trip cost is amortised over the load. Charnov's marginal value theorem says a forager should leave a patch when its marginal rate falls to the average for the habitat with travel counted as dead time, whose counterintuitive consequence is that longer travel means staying longer.

## Decision

**What a trip lifts is what the citizen brings multiplied by what the ground gives.** The second factor is the patch's own richness, not the distance the hauler walked, and that is a departure from §14's phrasing which the section itself licenses: it calls the load-by-distance evidence "a strong tendency and not a law -- at least one study found no significant load-size-by-distance interaction -- so this is a knob and not a certainty", and richness is the weaker claim of the two. It is also the better mechanism. Richness produces the distance effect endogenously, because far ground is richer and near ground is worked down, without a term that would pay a hauler for walking away from good ground near the hearth to poor ground further out -- which is what a raw distance term does and which no forager does. Richness is flat inside the old rim, so the near ring stays exactly as it was tuned. The property that what a citizen carries rises with no step in it holds on the richness axis as well as on the citizen's, and jointly.

**A hauler works a patch until its marginal take falls to the trip's average with travel counted as dead time.** Patch time becomes a thing that exists: a hauler takes a bite each tick rather than a whole load in one, the bite falls as the patch goes down, and they turn for home at the crossing. The theorem's currency is energy and ours is warmth, and the substitution is made literal rather than asserted: patch time and travel time are both priced in the warmth they cost, and warmth costs more the further from the fire it is spent, so the dead time on a far trip is dearer, the average it is measured against is lower, and the give-up point comes later. A distant treeline is therefore stripped further than a near one before anybody turns back, which is what makes far ground worth having.

## Consequences

The interior optimum ADR 0016 asserted becomes real and becomes testable, and the comment on `RICHNESS_BEST` that records its absence becomes false and goes in the same commit that falsifies it.

Haulers are away from the fire longer, which is the leash's business. ADR 0017's expectations and warmth's marks were tuned against trips that spent a single tick at the patch. If the death rate moves, that is a finding about the leash and it is reported as one; it is not absorbed by moving the marks, because a mark that moves to accommodate a change nobody measured is how a tuning layer stops meaning anything.

Trip counts stop being a measure of anything. More time at the patch means fewer trips carrying more, so flow is read per hand and by distance band, and any figure quoted in trips is a figure about the give-up rule rather than about the colony.
