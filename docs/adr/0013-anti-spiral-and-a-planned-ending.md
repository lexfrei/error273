# 0013. Anti-spiral measures and a planned ending

- Status: accepted, in part deferred
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

In a zero-player game an unrecoverable spiral is not a hard-won loss, it is several minutes of watching a spreadsheet die, because the watcher cannot intervene at all. The opposite failure is just as documented: a sandbox that never ends collapses into whatever gives the most immediate feedback.

## Decision

Crisis severity is a budget computed from population and stockpiles rather than rolled, held down through the first year and eased by a factor that rises with time since the last death and drops when one occurs. After a winter that kills a large share of the colony, the baseline need thresholds fall for the survivors for several seasons, so a reduced colony needs less to stay stable. A run has two reachable outcomes: colony death, recorded as a chronicle chapter with an epitaph rather than a game-over screen, and a survival end that arrives with the seasons layer.

## Consequences

Blunting the spiral costs some of the tragedy the setting is built on, and that trade is made deliberately because the watcher has no way to intervene. This layer ships before the mechanics that make spirals possible, not after them.

## Amendment, 2026-08-25, on what was built

**The budget landed as written and its ceiling is a measured number rather than a stated one.** Severity comes from population and stores per head, times the first-year ramp the winters already used, times a grace that rises with quiet time and drops when somebody dies. What it buys is spells of weather with an onset, so the air shows the approach rather than only the aftermath. Over a thousand spells the deepest reached the cap exactly and the worst single one took 14.9% of the colony's people, 5.6% of its fuel and 26.8% of its food -- enough to finish a colony that is already marginal, not enough to erase a healthy one. The figure for people is nearly three times what it was before the focus knee was calibrated against this same world: the lighter knee makes a hurt colony work worse in the ordinary range, so the same weather costs more lives. Depth, fuel and food did not move.

**Expectations fall after any season that takes a share, not only after a winter.** This is a departure from the decision above and it was forced by measurement. Across five worlds the worst winter took five per cent of a colony and the worst autumn took twenty-seven: the cold arrives while the colony is still working to a summer pattern, and by the time winter is deep whoever could not bear it has already gone. Keyed to winters the rule never fired once in nearly four hundred years of running. Keyed to seasons it fires as intended and rarely -- one season in thirty-one on the world that had that autumn, sparing twenty-two of thirty-nine survivors.

**The lever is on the mood and never on a survival mark.** A survivor feels every bad thought less for four seasons; what is going right is worth the same to them as to anybody. Lowering the mark a citizen sets out for warmth at would send them out later and kill them, which is the opposite of a mitigation.

**The planned ending is not in this layer.** Colony death is still an exit and not a chapter, and the survival end waits with it: both are the chronicle's, and this ADR is accepted for its anti-spiral half alone.
