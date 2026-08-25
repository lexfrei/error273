# 0002. Nested game clocks

- Status: accepted
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

Citizen lifecycles are naturally measured in years, construction in worker-days, and needs like warmth and rest in hours. A single flat clock forces awkward conventions to express all three at once.

## Decision

A tick is one game hour, with 24 hours per day, 30 days per season, and 4 seasons per year; the constants are centralized in one place. Every rate constant is defined in game-time units and converted at the edges, never expressed as a raw per-tick number. Real-time tempo (milliseconds per tick) is a separate, independently tunable knob. Movement stays one cell per tick, so a cell is treated as a unit of work rather than a physical distance, and the resulting abstraction of walking speed is accepted deliberately.

## Consequences

Lifecycle and seasonal mechanics can be built against the calendar later without a redesign of the clock, and balance tuning happens in human units (hours, days, seasons) instead of raw tick counts.

## Amendment, 2026-08-25

Ticks hold an absolute wall-clock grid, and the tempo knob this ADR reserved is exposed at launch as `ERROR273_TICK_MS`, clamped to a sane range and defaulting to 80. One tick is still one game hour whatever it is set to: the knob moves real time only, and no rate constant knows it exists.

Absolute means tick `n` falls due `n` steps after the colony was founded, not one step after whatever the previous tick happened to finish. Bevy's own `ScheduleRunnerPlugin` subtracts the frame's compute time from the wait, which is most of the way there, but it measures before sleeping and nothing ever takes back a sleep that overshoots, so a long run drifts by the sum of its own sleeps. The headless build therefore keeps its own runner, recomputing what is owed from the founding instant, with a bounded burst so a machine coming out of sleep catches up over several wakings rather than running an hour of colony in one frozen frame. The windowed build needs no such thing: `Time<Fixed>` accumulates real time and runs the fixed schedule as many times as fit, which is the same grid arrived at from the other direction, so it was left alone.
