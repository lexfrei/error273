# 0002. Nested game clocks

- Status: accepted
- Date: 2026-08-25

## Context

Citizen lifecycles are naturally measured in years, construction in worker-days, and needs like warmth and rest in hours. A single flat clock forces awkward conventions to express all three at once.

## Decision

A tick is one game hour, with 24 hours per day, 30 days per season, and 4 seasons per year; the constants are centralized in one place. Every rate constant is defined in game-time units and converted at the edges, never expressed as a raw per-tick number. Real-time tempo (milliseconds per tick) is a separate, independently tunable knob. Movement stays one cell per tick, so a cell is treated as a unit of work rather than a physical distance, and the resulting abstraction of walking speed is accepted deliberately.

## Consequences

Lifecycle and seasonal mechanics can be built against the calendar later without a redesign of the clock, and balance tuning happens in human units (hours, days, seasons) instead of raw tick counts.
