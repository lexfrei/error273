# 0016. The world has no edge

- Status: proposed
- Date: 2026-08-25

## Context

The disc of radius 18 is a renderer artefact rather than a boundary — nothing in the simulation clamps a position or asks whether a cell is on the map — but removing it puts a second axis under the tempo budget, which until now was written entirely in citizens because the cell count was a constant.

## Decision

The world is generated in 64-cell-square chunks, each a pure function of the world seed and its own coordinates, so chunks may be realised in any order and a run still replays; no placement rule may read a neighbouring chunk, which is the failure that produced cascading world generation elsewhere, and nothing generated lazily may draw from the order-dependent lineage counter. Storage is a hybrid: a patch the colony has touched keeps its delta permanently, while an untouched cell always regenerates from the seed, so a colony cannot cut a treeline out, walk away and return to a full one. Biomes are a lookup in a two-axis plane whose first axis is one generated field and whose second is distance from the hearth, since a single global climate collapses the temperature axis the ecological original uses; richness rises with that distance while the warmth cost of reaching it rises linearly, giving each trip an interior optimum. The budget gains a fourth rule to match: nothing is O(cells) or O(total patches) per tick, every gathering query is bounded by a search radius over a per-chunk index, regrowth runs over realised chunks only, and any flow field is computed over a bounded tile.

## Consequences

Chunk management is a one-time cost rather than a live system at this chunk size, and the per-chunk index earns its place by bounding queries rather than by bounding memory. A bound of order 2^20 cells is asserted by the headless instrument and relied on by nothing else — unbounded in play, bounded in proof — which also keeps the single-precision heat path orders of magnitude inside the range where integer coordinates stay exact. There is no board left to draw, so ADR 0014's fixed grid becomes a viewport.
