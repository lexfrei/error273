use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

const R: i32 = 18;
const CENTER: IVec2 = IVec2::new(R, R);
const AMBIENT: f32 = -30.0;
const GENERATOR_HEAT: f32 = 66.0;
const HEAT_FALLOFF: f32 = 3.0;
const BURN_EVERY: u64 = 4;
const CITIZENS: usize = 30;
const FOREST_CELLS: usize = 8;
const WOOD_PER_CELL: u32 = 40;
// Hysteresis: go warm up below the low mark, head back to work above the high
// one. A single threshold makes citizens oscillate at the edge of the warm zone.
const WARMTH_LOW: f32 = 25.0;
const WARMTH_HIGH: f32 = 75.0;

#[derive(Resource, Default)]
struct Tick(u64);

#[derive(Resource)]
struct Generator {
    fuel: u32,
}

#[derive(Resource)]
struct Forest(Vec<(IVec2, u32)>);

#[derive(Component)]
struct Pos(IVec2);

#[derive(Component)]
struct Citizen {
    warmth: f32,
    carrying: bool,
    warming: bool,
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(80))))
        .init_resource::<Tick>()
        .insert_resource(Generator { fuel: 20 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (advance_tick, citizen_ai, burn_fuel, render).chain(),
        )
        .run();
}

fn ring_pos(radius: f32, angle: f32) -> IVec2 {
    CENTER
        + IVec2::new(
            (radius * angle.cos()).round() as i32,
            (radius * angle.sin()).round() as i32,
        )
}

fn heat_at(p: IVec2, generator_on: bool) -> f32 {
    if !generator_on {
        return AMBIENT;
    }
    let d = p.as_vec2().distance(CENTER.as_vec2());
    (GENERATOR_HEAT - d * HEAT_FALLOFF).max(0.0) + AMBIENT
}

fn step_toward(from: IVec2, to: IVec2) -> IVec2 {
    from + (to - from).signum()
}

fn setup(mut commands: Commands) {
    print!("\x1B[2J");
    for i in 0..CITIZENS {
        let angle = i as f32 / CITIZENS as f32 * std::f32::consts::TAU;
        commands.spawn((
            Pos(ring_pos(2.0, angle)),
            Citizen {
                warmth: 80.0,
                carrying: false,
                warming: false,
            },
        ));
    }
    let cells = (0..FOREST_CELLS)
        .map(|i| {
            let angle = i as f32 / FOREST_CELLS as f32 * std::f32::consts::TAU;
            (ring_pos((R - 1) as f32, angle), WOOD_PER_CELL)
        })
        .collect();
    commands.insert_resource(Forest(cells));
}

fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

fn burn_fuel(tick: Res<Tick>, mut generator: ResMut<Generator>) {
    if tick.0.is_multiple_of(BURN_EVERY) {
        generator.fuel = generator.fuel.saturating_sub(1);
    }
}

fn citizen_ai(
    mut commands: Commands,
    mut generator: ResMut<Generator>,
    mut forest: ResMut<Forest>,
    mut citizens: Query<(Entity, &mut Pos, &mut Citizen)>,
) {
    for (entity, mut pos, mut citizen) in &mut citizens {
        let heat = heat_at(pos.0, generator.fuel > 0);
        citizen.warmth = if heat >= 0.0 {
            (citizen.warmth + 2.0).min(100.0)
        } else {
            citizen.warmth - 1.0
        };
        if citizen.warmth <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        if citizen.warmth < WARMTH_LOW {
            citizen.warming = true;
        } else if citizen.warmth >= WARMTH_HIGH {
            citizen.warming = false;
        }

        let near_center = (pos.0 - CENTER).abs().max_element() <= 1;
        if citizen.carrying && near_center {
            generator.fuel += 1;
            citizen.carrying = false;
        }

        let target = if citizen.warming || citizen.carrying {
            CENTER
        } else {
            match nearest_wood(&forest, pos.0) {
                Some(cell) => {
                    if pos.0 == cell {
                        take_wood(&mut forest, cell);
                        citizen.carrying = true;
                        CENTER
                    } else {
                        cell
                    }
                }
                None => CENTER,
            }
        };
        if pos.0 != target {
            pos.0 = step_toward(pos.0, target);
        }
    }
}

fn nearest_wood(forest: &Forest, from: IVec2) -> Option<IVec2> {
    forest
        .0
        .iter()
        .filter(|(_, wood)| *wood > 0)
        .min_by_key(|(cell, _)| (*cell - from).abs().max_element())
        .map(|(cell, _)| *cell)
}

fn take_wood(forest: &mut Forest, cell: IVec2) {
    if let Some((_, wood)) = forest.0.iter_mut().find(|(c, _)| *c == cell) {
        *wood -= 1;
    }
}

fn render(
    tick: Res<Tick>,
    generator: Res<Generator>,
    forest: Res<Forest>,
    citizens: Query<(&Pos, &Citizen)>,
) {
    let size = (R * 2 + 1) as usize;
    let mut grid = vec![vec![' '; size]; size];
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let p = IVec2::new(x as i32, y as i32);
            if p.as_vec2().distance(CENTER.as_vec2()) <= R as f32 + 0.5 {
                *cell = '.';
            }
        }
    }
    for (cell, wood) in &forest.0 {
        grid[cell.y as usize][cell.x as usize] = if *wood > 0 { 'T' } else { 't' };
    }
    for (pos, citizen) in &citizens {
        grid[pos.0.y as usize][pos.0.x as usize] = if citizen.carrying { 'W' } else { '@' };
    }
    grid[CENTER.y as usize][CENTER.x as usize] = '#';

    let alive = citizens.iter().count();
    let wood_left: u32 = forest.0.iter().map(|(_, w)| w).sum();
    let mut out = String::from("\x1B[H");
    for row in &grid {
        out.push_str(&row.iter().collect::<String>());
        out.push('\n');
    }
    out.push_str(&format!(
        "tick {:5}  citizens {:2}  fuel {:3}  forest {:3}\n",
        tick.0, alive, generator.fuel, wood_left
    ));
    print!("{out}");

    if alive == 0 {
        println!("Everyone froze. The city is silent.");
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_falls_off_with_distance() {
        let near = heat_at(CENTER + IVec2::new(1, 0), true);
        let far = heat_at(CENTER + IVec2::new(10, 0), true);
        assert!(near > far);
        assert!(heat_at(CENTER, true) > 0.0);
    }

    #[test]
    fn heat_is_ambient_when_generator_is_off() {
        assert_eq!(heat_at(CENTER, false), AMBIENT);
        assert_eq!(heat_at(CENTER + IVec2::new(5, 5), false), AMBIENT);
    }

    #[test]
    fn step_toward_reduces_distance_and_stops_at_target() {
        let target = IVec2::new(10, 3);
        let mut p = IVec2::new(0, 0);
        let mut dist = (target - p).abs().max_element();
        while p != target {
            p = step_toward(p, target);
            let next = (target - p).abs().max_element();
            assert!(next < dist);
            dist = next;
        }
        assert_eq!(step_toward(p, target), p + (target - p).signum());
    }
}
