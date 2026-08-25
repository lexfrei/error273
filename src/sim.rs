use bevy::prelude::*;

pub const R: i32 = 18;
pub const CENTER: IVec2 = IVec2::new(R, R);
pub const AMBIENT: f32 = -30.0;
pub const GENERATOR_HEAT: f32 = 66.0;
pub const HEAT_FALLOFF: f32 = 3.0;
pub const BURN_EVERY: u64 = 4;
pub const CITIZENS: usize = 30;
pub const FOREST_CELLS: usize = 8;
pub const WOOD_PER_CELL: u32 = 40;
// Hysteresis: go warm up below the low mark, head back to work above the high
// one. A single threshold makes citizens oscillate at the edge of the warm zone.
pub const WARMTH_LOW: f32 = 25.0;
pub const WARMTH_HIGH: f32 = 75.0;

#[derive(Resource, Default)]
pub struct Tick(pub u64);

#[derive(Resource)]
pub struct Generator {
    pub fuel: u32,
}

#[derive(Resource)]
pub struct Forest(pub Vec<(IVec2, u32)>);

#[derive(Component)]
pub struct Pos(pub IVec2);

#[derive(Component)]
pub struct Citizen {
    pub warmth: f32,
    pub carrying: bool,
    pub warming: bool,
}

pub fn ring_pos(radius: f32, angle: f32) -> IVec2 {
    CENTER
        + IVec2::new(
            (radius * angle.cos()).round() as i32,
            (radius * angle.sin()).round() as i32,
        )
}

pub fn heat_at(p: IVec2, generator_on: bool) -> f32 {
    if !generator_on {
        return AMBIENT;
    }
    let d = p.as_vec2().distance(CENTER.as_vec2());
    (GENERATOR_HEAT - d * HEAT_FALLOFF).max(0.0) + AMBIENT
}

pub fn step_toward(from: IVec2, to: IVec2) -> IVec2 {
    from + (to - from).signum()
}

pub fn setup(mut commands: Commands) {
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

pub fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

pub fn burn_fuel(tick: Res<Tick>, mut generator: ResMut<Generator>) {
    if tick.0.is_multiple_of(BURN_EVERY) {
        generator.fuel = generator.fuel.saturating_sub(1);
    }
}

pub fn citizen_ai(
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

pub fn nearest_wood(forest: &Forest, from: IVec2) -> Option<IVec2> {
    forest
        .0
        .iter()
        .filter(|(_, wood)| *wood > 0)
        .min_by_key(|(cell, _)| (*cell - from).abs().max_element())
        .map(|(cell, _)| *cell)
}

pub fn take_wood(forest: &mut Forest, cell: IVec2) {
    if let Some((_, wood)) = forest.0.iter_mut().find(|(c, _)| *c == cell) {
        *wood -= 1;
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
