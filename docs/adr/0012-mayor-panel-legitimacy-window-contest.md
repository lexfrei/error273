# 0012. The mayor's panel: legitimacy, window, and contest

- Status: proposed
- Date: 2026-08-25

## Context

ADR 0004 makes the mayor a data knob; what is missing is the resource that scales it and the machinery that pushes back. Two failures are well documented elsewhere: a legitimacy stat with one reliable source degenerates into farming that source, and contestation that fires on a timer teaches the watcher that the colony's state is noise.

## Decision

Legitimacy is a slowly refilling budget that both multiplies the mayor's vote weight and sets the probability a directive is obeyed, fed by three independent and non-spammable inflows: surviving a named crisis, granting a petition, and delivering a promise made to win a vote. The set of proposable policies is gated by the colony's own position on the culture axes, the only override is earned by service to the objectors rather than bought with a generic currency, and repealing a policy soon after enacting it raises a colony cynicism that caps legitimacy gain until it ages out. The mayor's one economic lever is a firewood ration per workplace, which biases job scoring; every petition names a buildable or enactable thing with a visible cost, and denying one imprints a tradition card on the petitioners. At legitimacy zero a grace season opens with a concrete demand and a named scapegoat option rather than an immediate deposition, and contestation escalates through petition, work-to-rule, quiet sabotage, council override and exile, each stage exposing a de-escalation that costs a different resource than the one under pressure.

## Consequences

Colony unrest sets how large and how often contestation fires while the participants are drawn separately from the carriers of the offended card, so scale and authorship can be tuned apart. An ignored directive always emits a chronicle line naming why, without which an unreliable lever reads as a bug rather than as politics.
