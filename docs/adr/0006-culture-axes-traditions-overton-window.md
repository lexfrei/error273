# 0006. Culture as axes, traditions as carried cards, policies gated by an Overton window

- Status: proposed
- Date: 2026-08-25

## Context

The mayor must not be omnipotent: policies with eugenics-like implications should only become reachable after generations of genuine cultural change, not on a whim.

## Decision

Culture is modeled as several independent axes (collectivist-individualist, pious-secular, harsh-lenient) rather than a single scalar. Every policy has a position on these axes and can only be enacted while it falls inside the colony's current window; outside it, the policy is refused with protest and a loss of legitimacy. The window itself shifts slowly, driven by entertainers, schools, traditions, and edge policies. Legitimacy is a mayor resource that multiplies the vote bias. Traditions are colony modifiers carried by individual citizens, transmitted at birth and by contact, dying out with their last carrier, and sometimes born from events; harsh eradication policies cost mood and can imprint new traditions in response.

## Consequences

Play is oriented around a long horizon, and the culture layer is load-bearing for what the mayor can do rather than a cosmetic overlay.
