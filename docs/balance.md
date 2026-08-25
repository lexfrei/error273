# Balance log

One row per commit that can move the shape of a run, so a tuning change that quietly flattens the arc is visible as a diff rather than a memory.

Rows are keyed by commit subject rather than hash: a commit cannot contain its own hash, and a rebase invalidates one written after the fact.

| commit | change | 15 s status | peak pop (tick) | forest 0 (tick) | fuel 0 (tick) | last death (tick) | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| feat: let the colony grow at the cost of its firewood | growth loop | `tick 166 pop 38 houses 13 build - fuel 57 forest 155` | 49 (296) | ~350 | ~390 | 518 | first arc with births; SHELTER_DRAIN_FACTOR 0.4 to 0.7 shortened a 200-tick flat tail |
| fix: keep timber that arrives after a house is finished | surplus timber fix | `tick 165 pop 38 houses 13 build - fuel 62 forest 157` | 51 (312) | ~355 | ~390 | 519 | logs past a finished house now burn instead of vanishing, which bought two more citizens and one more house |
| feat: put the simulation on a game calendar | game calendar | `tick 165 year 1 Spring day 7 hour 21 / pop 38 houses 13 build - fuel 62 forest 157` | 51 (312) | ~355 | ~390 | 519 | units only: rates restated per game hour and day at unchanged effective values |
| feat: give citizens hunger and one shape for every need | hunger and the needs vector | `tick 166 year 1 Spring day 7 hour 22 / pop 38 houses 13 build - fuel 62 food 30 wood 158 game 240` | 42 (248) | not reached, 9 left | 364 | 463 | hauling now splits between wood and game, so the arc is shorter and thinner; FOOD decay per_day(14.0) starved the colony outright and was cut to per_day(7.0), with FUEL_PER_CITIZEN 1.3 and FOOD_PER_CITIZEN 0.6 setting which stockpile a hauler judges shorter |
