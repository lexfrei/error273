# 0014. A windowed ASCII renderer

- Status: accepted
- Date: 2026-08-25
- Amended: 2026-08-25

## Context

A terminal character box is roughly 1:2, so the map's disc of radius 18 reaches the screen as an oval, and a cell has nowhere to carry anything that is not a glyph. ADR 0001 picked the terminal knowing the renderer was one replaceable system among many; this is the replacement, and the heat map is the first thing it can show that the terminal could not.

## Decision

An optional `window` cargo feature adds a Bevy 2D renderer: the same glyphs on 22 px square cells, each standing on a background quad tinted by `heat_at`, with the four status lines underneath. The world of ADR 0016 has no edge, so the renderer draws a viewport rather than a board — a fixed window of cells locked to the hearth, repainted at a cost set by the viewport and not by the world. Citizens walk off the frame and the status lines report how many are out. The colony runs from `FixedUpdate` on the same 80 ms step the headless run loop uses, so both builds tick at one rate and no simulation code knows which renderer is watching. The feature enables `default_platform` and `2d_bevy_render` rather than Bevy's curated `2d` profile, which would also pull in scenes and picking that nothing here uses. The face is JetBrains Mono — monospaced, openly licensed, and unambiguous between similar glyphs at the size a cell allows — vendored under `assets/fonts/` with its OFL 1.1 license and compiled into the binary rather than read back at runtime, so the map draws the same wherever it is started from.

## Consequences

The whole colony can no longer be seen at once, and the warm ring is the first thing to reach past the frame; that is the price of the world having no edge, and a view that follows an expedition is queued for a later iteration of the window rather than solved here. The default build is untouched — no window, no GPU, the same dependencies and the same build time — and the terminal renderer stands down rather than being removed. Contrast is the renderer's responsibility, following Brogue's rule that a glyph stays readable whatever its background does, so every ink is lifted until it clears its own cell by a fixed margin of luminance. The glyph table and the status lines now exist twice, once per renderer; that is what keeping the two independent costs, and it is the thing to unify if a third one ever appears.

## Amendment, 2026-08-25

The window is the default build and the terminal map renderer is gone. Building with `--no-default-features` leaves the headless instrument: the same simulation on the same step, printing the status lines to stdout and drawing no map, which is what the quality gates and the balance log read. That mode also honours `ERROR273_TURBO`, which drops the wait between ticks so a run long enough to see a citizen grow up takes minutes rather than hours.

Both duplications this ADR recorded are closed rather than deferred. The glyph table went with the terminal map, so the window's marks are now the only picture of the colony. The status lines moved into one module that both faces fill in and format through, which makes a reading that exists in one and not the other a compile error -- the drift had already happened once, since the window never showed the air temperature the terminal added.

## Amendment, 2026-08-25, on the frame

The viewport this ADR described is built, and two of its numbers moved on contact. The frame is the twenty-four cells around the hearth that a citizen looks for work in, not the eighteen the old disc held, because a frame that stopped short of the working ring would hide the thing a watcher is trying to see. That is 49 cells across rather than 37, so the cell went from 22 px to 18 to keep the whole window inside a laptop display; the glyph went with it and the ratio between them is unchanged.

The disc went with the board. Every cell in the frame now carries the heat map, where before the outside of the disc was painted flat to make the round shape read -- there is no outside any more, only ground the frame does not reach. What is off the frame is not drawn at all, and the count of citizens out there is a reading in the shared status structure, so the headless build reports it too. That build has no frame, which makes the number mean something slightly different there: not "cannot be shown" but "working past the near ring". Both are the same question about the same colony, which is why it is one field.
