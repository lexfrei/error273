//! A windowed renderer that draws the same ASCII the terminal draws, on square
//! cells. A terminal character box is about 1:2, which stretches the colony's
//! disc into an oval; here a cell is a square, so the disc is round.
//!
//! Nothing in here writes to the simulation. It reads the same resources and
//! components the terminal renderer reads, and the sim runs on the same fixed
//! 80 ms step it runs on headless.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::WindowResolution;

use crate::sim::{
    Air, BUILDINGS, Building, CENTER, Cargo, Citizen, Construction, GENERATOR_HEAT, NeedKind,
    Patches, Pos, Regard, STAT_COUNT, STATS, Structure, VIEW_RADIUS, estimate, focus_band,
    focus_of, hardship_status, is_known, median, on_frame, regard_of,
};
use crate::status::{CitizenCard, Readings, STATUS_LINES, Status, status_lines};

/// Compiled in rather than loaded from `assets/`, so the binary draws the same
/// map wherever it is run from.
const FACE: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");

const GRID: usize = (VIEW_RADIUS * 2 + 1) as usize;
/// Square, which is the whole point of this renderer.
const CELL: f32 = 18.0;
const MAP_SPAN: f32 = GRID as f32 * CELL;

const GLYPH_SIZE: f32 = 15.0;
const STATUS_SIZE: f32 = 15.0;
const STATUS_LEAD: f32 = 22.0;
const MARGIN: f32 = 14.0;

const STATUS_SPAN: f32 = STATUS_LEAD * STATUS_LINES as f32;
const WINDOW_W: f32 = MAP_SPAN + MARGIN * 2.0;
const WINDOW_H: f32 = MAP_SPAN + STATUS_SPAN + MARGIN * 2.0;
/// The map is centred on the origin, so the camera drops by half the status
/// block to leave room for it underneath.
const CAMERA_Y: f32 = -STATUS_SPAN / 2.0;

/// The heat ramp, walked from ambient to a fully fed generator. Kept dark on
/// purpose: these are backgrounds, and the glyphs have to stay legible on them.
const FROST: Srgba = Srgba::rgb(0.03, 0.04, 0.08);
const EMBER: Srgba = Srgba::rgb(0.22, 0.10, 0.06);
const HEARTH: Srgba = Srgba::rgb(0.48, 0.21, 0.06);
const VOID: Color = Color::srgb(0.01, 0.01, 0.02);

/// How far a glyph must sit above its background in luminance. Contrast is the
/// renderer's job, not the palette author's: every ink is lifted to clear this.
const MIN_CONTRAST: f32 = 0.25;

const INK_STATUS: Color = Color::srgb(0.78, 0.80, 0.86);

/// What one cell shows: the character the terminal would print, and the ink to
/// print it in.
#[derive(Clone, Copy)]
struct Mark {
    glyph: char,
    ink: Color,
}

const FLOOR: Mark = Mark {
    glyph: '.',
    ink: Color::srgb(0.30, 0.32, 0.38),
};
const SITE: Mark = Mark {
    glyph: '+',
    ink: Color::srgb(0.95, 0.80, 0.35),
};
const GENERATOR: Mark = Mark {
    glyph: '#',
    ink: Color::srgb(1.00, 0.86, 0.56),
};

/// Which cell of the map an entity draws, in the same coordinates the sim uses.
#[derive(Component, Clone, Copy)]
struct Cell(IVec2);

/// One of the status lines under the map, numbered from the top.
#[derive(Component)]
struct StatusLine(usize);

pub struct WindowRendererPlugin;

impl Plugin for WindowRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "error273".into(),
                resolution: WindowResolution::new(WINDOW_W as u32, WINDOW_H as u32),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(VOID))
        // `Time<Fixed>` accumulates real time and runs the fixed schedule as many
        // times as fit, so a slow frame is made up rather than lost. All the
        // window needs from the tempo knob is the step itself.
        .insert_resource(Time::<Fixed>::from_duration(crate::tempo::tick_step()))
        .add_systems(Startup, spawn_board)
        .add_systems(FixedPostUpdate, (paint_map, paint_status));
    }
}

fn spawn_board(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(FACE.to_vec()));
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            // The board never gets cropped, whatever the window and the display
            // density turn out to be.
            scaling_mode: ScalingMode::AutoMin {
                min_width: WINDOW_W,
                min_height: WINDOW_H,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, CAMERA_Y, 0.0),
    ));
    for y in 0..GRID {
        for x in 0..GRID {
            let cell = Cell(CENTER + IVec2::new(x as i32, y as i32) - IVec2::splat(VIEW_RADIUS));
            let at = world_of(cell.0);
            commands.spawn((
                cell,
                Sprite::from_color(VOID, Vec2::splat(CELL)),
                Transform::from_xyz(at.x, at.y, 0.0),
            ));
            commands.spawn((
                cell,
                Text2d::new(" "),
                TextFont::from_font_size(GLYPH_SIZE).with_font(font.clone()),
                TextColor(VOID),
                Transform::from_xyz(at.x, at.y, 1.0),
            ));
        }
    }
    for line in 0..STATUS_LINES {
        commands.spawn((
            StatusLine(line),
            Text2d::new(""),
            TextFont::from_font_size(STATUS_SIZE).with_font(font.clone()),
            TextColor(INK_STATUS),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(
                -MAP_SPAN / 2.0,
                -MAP_SPAN / 2.0 - STATUS_LEAD * (line as f32 + 0.5),
                1.0,
            ),
        ));
    }
}

fn paint_map(
    air: Res<Air>,
    patches: Res<Patches>,
    construction: Res<Construction>,
    structures: Query<(&Pos, &Structure)>,
    citizens: Query<(&Pos, &Citizen)>,
    mut tiles: Query<(&Cell, &mut Sprite)>,
    mut glyphs: Query<(&Cell, &mut Text2d, &mut TextColor)>,
) {
    let mut grid = vec![vec![FLOOR; GRID]; GRID];
    // The frame is a window on a world that does not stop at its edges, so
    // anything standing outside it is simply not painted. The status line says
    // how many citizens that is.
    let mark_at = |grid: &mut Vec<Vec<Mark>>, at: IVec2, mark: Mark| {
        if on_frame(at) {
            let framed = at - CENTER + IVec2::splat(VIEW_RADIUS);
            grid[framed.y as usize][framed.x as usize] = mark;
        }
    };
    for patch in patches.seen(CENTER, VIEW_RADIUS) {
        mark_at(&mut grid, patch.pos, patch_mark(patch.kind, patch.amount));
    }
    for (pos, structure) in &structures {
        mark_at(&mut grid, pos.0, structure_mark(structure.0));
    }
    if let Some(site) = &construction.site {
        mark_at(&mut grid, site.pos, SITE);
    }
    for (pos, citizen) in &citizens {
        mark_at(&mut grid, pos.0, citizen_mark(citizen, pos.0));
    }
    mark_at(&mut grid, CENTER, GENERATOR);

    let air = &*air;
    for (cell, mut sprite) in &mut tiles {
        let ground = ground_of(cell.0, air);
        if sprite.color != ground {
            sprite.color = ground;
        }
    }
    for (cell, mut text, mut ink) in &mut glyphs {
        let framed = cell.0 - CENTER + IVec2::splat(VIEW_RADIUS);
        let mark = grid[framed.y as usize][framed.x as usize];
        if !text.starts_with(mark.glyph) {
            text.clear();
            text.push(mark.glyph);
        }
        let readable = lift(mark.ink, ground_of(cell.0, air));
        if ink.0 != readable {
            ink.0 = readable;
        }
    }
}

fn paint_status(
    readings: Readings,
    structures: Query<&Structure>,
    citizens: Query<&Citizen>,
    walkers: Query<&Pos, With<Citizen>>,
    mut lines: Query<(&StatusLine, &mut Text2d)>,
) {
    let mut buildings = [0usize; BUILDINGS.len()];
    for structure in &structures {
        buildings[structure.0 as usize] += 1;
    }
    let ages: Vec<f32> = citizens.iter().map(|citizen| citizen.age).collect();
    let mut stats = [0.0; STAT_COUNT];
    for stat in STATS {
        let mut held: Vec<f32> = citizens
            .iter()
            .map(|citizen| citizen.upbringing.stats().of(stat))
            .collect();
        stats[stat as usize] = median(&mut held);
    }
    let mut moods: Vec<f32> = citizens.iter().map(|citizen| citizen.mood).collect();
    let mood = median(&mut moods);
    let card = citizens
        .iter()
        .max_by(|a, b| a.age.total_cmp(&b.age))
        .map(|citizen| {
            let mut words = [Regard::Middling; STAT_COUNT];
            for stat in STATS {
                let guess = estimate(
                    citizen.upbringing.stats().of(stat),
                    citizen.upbringing.prosperity(),
                    citizen.watched,
                );
                words[stat as usize] = regard_of(guess, stats[stat as usize]);
            }
            CitizenCard {
                focus: focus_band(focus_of(&citizen.needs)),
                mood: citizen.mood,
                held: citizen.held,
                hardship: hardship_status(citizen.hardship),
                seed: citizen.seed,
                age: citizen.age,
                words,
                watched: citizen.watched,
                known: is_known(citizen.watched),
            }
        });
    let status = Status {
        tick: readings.outside.tick.0,
        calendar: *readings.outside.calendar,
        ambient: readings.outside.air.ambient,
        alive: ages.len(),
        off_frame: walkers.iter().filter(|pos| !on_frame(pos.0)).count(),
        missing: readings.missing.count(),
        fuel: readings.stores.generator.fuel,
        food: readings.stores.granary.food,
        wood: readings.standing(Cargo::Wood),
        game: readings.standing(Cargo::Food),
        buildings,
        project: readings
            .construction
            .site
            .as_ref()
            .map(|site| (site.building, site.delivered)),
        tally: readings.ballot.tally,
        ages,
        stats,
        mood,
        card,
    };
    let painted = status_lines(&status);
    for (line, mut text) in &mut lines {
        if text.0 != painted[line.0] {
            text.0.clone_from(&painted[line.0]);
        }
    }
}

/// The sim counts rows downwards from the top; the screen counts upwards. The
/// frame is locked to the hearth, so a cell's place on screen is its offset
/// from it.
fn world_of(p: IVec2) -> Vec2 {
    let from_hearth = p - CENTER;
    Vec2::new(from_hearth.x as f32 * CELL, -from_hearth.y as f32 * CELL)
}

/// Warmth as a background. Every cell in the frame gets the heat map now: there
/// is no outside to leave flat, only ground the frame does not reach.
fn ground_of(p: IVec2, air: &Air) -> Color {
    let warmth = ((air.heat_at(p) - air.ambient) / GENERATOR_HEAT).clamp(0.0, 1.0);
    if warmth < 0.5 {
        FROST.mix(&EMBER, warmth * 2.0).into()
    } else {
        EMBER.mix(&HEARTH, (warmth - 0.5) * 2.0).into()
    }
}

/// Raises an ink until it clears its background by [`MIN_CONTRAST`], so a cell
/// that warms up cannot swallow the glyph standing on it.
fn lift(ink: Color, ground: Color) -> Color {
    let floor = ground.luminance() + MIN_CONTRAST;
    if ink.luminance() < floor {
        ink.with_luminance(floor)
    } else {
        ink
    }
}

fn patch_mark(kind: Cargo, amount: u32) -> Mark {
    match (kind, amount > 0) {
        (Cargo::Wood, true) => Mark {
            glyph: 'T',
            ink: Color::srgb(0.36, 0.72, 0.40),
        },
        (Cargo::Wood, false) => Mark {
            glyph: 't',
            ink: Color::srgb(0.30, 0.44, 0.32),
        },
        (Cargo::Food, true) => Mark {
            glyph: 'Y',
            ink: Color::srgb(0.86, 0.73, 0.30),
        },
        (Cargo::Food, false) => Mark {
            glyph: 'y',
            ink: Color::srgb(0.47, 0.42, 0.26),
        },
    }
}

fn structure_mark(building: Building) -> Mark {
    Mark {
        glyph: match building {
            Building::House => 'H',
            Building::HuntersHut => 'V',
            Building::GeneratorUpgrade => 'B',
            Building::Waystation => 'P',
        },
        ink: match building {
            Building::House => Color::srgb(0.73, 0.77, 0.83),
            Building::HuntersHut => Color::srgb(0.71, 0.81, 0.72),
            Building::GeneratorUpgrade => Color::srgb(0.87, 0.71, 0.55),
            Building::Waystation => Color::srgb(0.94, 0.62, 0.32),
        },
    }
}

fn citizen_mark(citizen: &Citizen, pos: IVec2) -> Mark {
    match citizen.carrying {
        Some(Cargo::Wood) => Mark {
            glyph: 'W',
            ink: Color::srgb(0.62, 0.87, 0.56),
        },
        Some(Cargo::Food) => Mark {
            glyph: 'F',
            ink: Color::srgb(0.96, 0.86, 0.46),
        },
        None if citizen.needs.get(NeedKind::Rest).pressing && pos == citizen.home => Mark {
            glyph: 'z',
            ink: Color::srgb(0.56, 0.63, 0.82),
        },
        None => Mark {
            glyph: '@',
            ink: Color::srgb(0.93, 0.95, 0.99),
        },
    }
}
