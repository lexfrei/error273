# 0014. A windowed ASCII renderer

- Status: accepted
- Date: 2026-08-25

## Context

A terminal character box is roughly 1:2, so the map's disc of radius 18 reaches the screen as an oval, and a cell has nowhere to carry anything that is not a glyph. ADR 0001 picked the terminal knowing the renderer was one replaceable system among many; this is the replacement, and the heat map is the first thing it can show that the terminal could not.

## Decision

An optional `window` cargo feature adds a Bevy 2D renderer: the same glyphs on 22 px square cells, each standing on a background quad tinted by `heat_at`, with the four status lines underneath. The colony runs from `FixedUpdate` on the same 80 ms step the headless run loop uses, so both builds tick at one rate and no simulation code knows which renderer is watching. The feature enables `default_platform` and `2d_bevy_render` rather than Bevy's curated `2d` profile, which would also pull in scenes and picking that nothing here uses. The face is JetBrains Mono — monospaced, openly licensed, and unambiguous between similar glyphs at the size a cell allows — vendored under `assets/fonts/` with its OFL 1.1 license and compiled into the binary rather than read back at runtime, so the map draws the same wherever it is started from.

## Consequences

The default build is untouched — no window, no GPU, the same dependencies and the same build time — and the terminal renderer stands down rather than being removed. Contrast is the renderer's responsibility, following Brogue's rule that a glyph stays readable whatever its background does, so every ink is lifted until it clears its own cell by a fixed margin of luminance. The glyph table and the status lines now exist twice, once per renderer; that is what keeping the two independent costs, and it is the thing to unify if a third one ever appears.
