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
pub const WARMTH_MAX: f32 = 100.0;
pub const WARMTH_GAIN: f32 = 2.0;
pub const WARMTH_DRAIN: f32 = 1.0;
// A house is shelter, not a stove: it only slows the bleed of body heat.
pub const SHELTER_DRAIN_FACTOR: f32 = 0.4;
// Hysteresis: go warm up below the low mark, head back to work above the high
// one. A single threshold makes citizens oscillate at the edge of the warm zone.
pub const WARMTH_LOW: f32 = 25.0;
pub const WARMTH_HIGH: f32 = 75.0;
pub const FATIGUE_MAX: f32 = 100.0;
pub const FATIGUE_GAIN: f32 = 1.0;
pub const FATIGUE_RECOVERY: f32 = 4.0;
// Same hysteresis idea as warmth, so nobody twitches on the doorstep.
pub const FATIGUE_HIGH: f32 = 60.0;
pub const FATIGUE_LOW: f32 = 10.0;
pub const HOUSE_CAPACITY: usize = 3;
pub const HOUSES_PER_RING: usize = 12;
pub const HOUSE_RING_START: i32 = 5;
pub const HOUSE_RING_STEP: i32 = 2;
// Keep a margin between the outermost house ring and the forest on the rim.
pub const HOUSE_MAX_RADIUS: i32 = R - 3;

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
pub struct House;

#[derive(Component)]
pub struct Citizen {
    pub warmth: f32,
    pub fatigue: f32,
    pub home: IVec2,
    pub carrying: bool,
    pub warming: bool,
    pub resting: bool,
}

/// What a citizen spends this tick on, highest priority first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duty {
    WarmUp,
    Deliver,
    Rest,
    Gather,
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

pub fn warmth_step(warmth: f32, heat: f32, sheltered: bool) -> f32 {
    if heat >= 0.0 {
        return (warmth + WARMTH_GAIN).min(WARMTH_MAX);
    }
    let drain = if sheltered {
        WARMTH_DRAIN * SHELTER_DRAIN_FACTOR
    } else {
        WARMTH_DRAIN
    };
    warmth - drain
}

pub fn update_warming(warming: bool, warmth: f32) -> bool {
    if warmth < WARMTH_LOW {
        true
    } else if warmth >= WARMTH_HIGH {
        false
    } else {
        warming
    }
}

pub fn fatigue_step(fatigue: f32, recovering: bool) -> f32 {
    if recovering {
        (fatigue - FATIGUE_RECOVERY).max(0.0)
    } else {
        (fatigue + FATIGUE_GAIN).min(FATIGUE_MAX)
    }
}

pub fn update_resting(resting: bool, fatigue: f32) -> bool {
    if fatigue >= FATIGUE_HIGH {
        true
    } else if fatigue <= FATIGUE_LOW {
        false
    } else {
        resting
    }
}

pub fn choose_duty(warming: bool, carrying: bool, resting: bool) -> Duty {
    if warming {
        Duty::WarmUp
    } else if carrying {
        // Drop the load off first, otherwise the wood sleeps in the house.
        Duty::Deliver
    } else if resting {
        Duty::Rest
    } else {
        Duty::Gather
    }
}

/// Fixed site for the n-th house, filling one ring before moving outward so
/// that adding a house never displaces the ones already standing.
pub fn house_site(index: usize) -> Option<IVec2> {
    let ring = index / HOUSES_PER_RING;
    let radius = HOUSE_RING_START + ring as i32 * HOUSE_RING_STEP;
    if radius > HOUSE_MAX_RADIUS {
        return None;
    }
    let slot = index % HOUSES_PER_RING;
    // Offset every other ring by half a slot so streets are not fully radial.
    let offset = if ring.is_multiple_of(2) { 0.0 } else { 0.5 };
    let angle = (slot as f32 + offset) / HOUSES_PER_RING as f32 * std::f32::consts::TAU;
    Some(ring_pos(radius as f32, angle))
}

pub fn forest_sites() -> Vec<IVec2> {
    (0..FOREST_CELLS)
        .map(|i| {
            let angle = i as f32 / FOREST_CELLS as f32 * std::f32::consts::TAU;
            ring_pos((R - 1) as f32, angle)
        })
        .collect()
}

/// First house with a free bed, given where everyone currently lives.
pub fn free_home(sites: &[IVec2], homes: &[IVec2]) -> Option<IVec2> {
    sites
        .iter()
        .find(|site| homes.iter().filter(|home| *home == *site).count() < HOUSE_CAPACITY)
        .copied()
}

pub fn setup(mut commands: Commands) {
    let houses = CITIZENS.div_ceil(HOUSE_CAPACITY);
    let sites: Vec<IVec2> = (0..houses).filter_map(house_site).collect();
    for site in &sites {
        commands.spawn((Pos(*site), House));
    }

    let mut homes: Vec<IVec2> = Vec::new();
    for i in 0..CITIZENS {
        let Some(home) = free_home(&sites, &homes) else {
            break;
        };
        homes.push(home);
        let angle = i as f32 / CITIZENS as f32 * std::f32::consts::TAU;
        commands.spawn((
            Pos(ring_pos(2.0, angle)),
            Citizen {
                warmth: 80.0,
                fatigue: 0.0,
                home,
                carrying: false,
                warming: false,
                resting: false,
            },
        ));
    }

    let cells = forest_sites()
        .into_iter()
        .map(|cell| (cell, WOOD_PER_CELL))
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
    let generator_on = generator.fuel > 0;
    for (entity, mut pos, mut citizen) in &mut citizens {
        let at_home = pos.0 == citizen.home;
        let heat = heat_at(pos.0, generator_on);
        citizen.warmth = warmth_step(citizen.warmth, heat, at_home);
        if citizen.warmth <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        citizen.warming = update_warming(citizen.warming, citizen.warmth);

        let near_center = (pos.0 - CENTER).abs().max_element() <= 1;
        if citizen.carrying && near_center {
            generator.fuel += 1;
            citizen.carrying = false;
        }

        let duty = choose_duty(citizen.warming, citizen.carrying, citizen.resting);
        citizen.fatigue = fatigue_step(citizen.fatigue, duty == Duty::Rest && at_home);
        citizen.resting = update_resting(citizen.resting, citizen.fatigue);

        let target = match duty {
            Duty::WarmUp | Duty::Deliver => CENTER,
            Duty::Rest => citizen.home,
            Duty::Gather => match nearest_wood(&forest, pos.0) {
                Some(cell) if pos.0 == cell => {
                    take_wood(&mut forest, cell);
                    citizen.carrying = true;
                    CENTER
                }
                Some(cell) => cell,
                None => CENTER,
            },
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

    #[test]
    fn fatigue_rises_while_working_and_falls_while_recovering() {
        let worked = fatigue_step(10.0, false);
        assert!(worked > 10.0);
        let rested = fatigue_step(10.0, true);
        assert!(rested < 10.0);
    }

    #[test]
    fn fatigue_stays_within_bounds() {
        assert_eq!(fatigue_step(FATIGUE_MAX, false), FATIGUE_MAX);
        assert_eq!(fatigue_step(0.0, true), 0.0);
    }

    #[test]
    fn resting_flips_only_at_the_hysteresis_thresholds() {
        let mid = (FATIGUE_LOW + FATIGUE_HIGH) / 2.0;
        assert!(
            !update_resting(false, mid),
            "tired-but-not-spent keeps working"
        );
        assert!(
            update_resting(true, mid),
            "half-rested citizen stays in bed"
        );
        assert!(update_resting(false, FATIGUE_HIGH));
        assert!(!update_resting(true, FATIGUE_LOW));
    }

    #[test]
    fn a_full_work_rest_cycle_switches_state_exactly_twice() {
        let mut fatigue = 0.0_f32;
        let mut resting = false;
        let mut flips = 0;
        let mut ticks = 0;
        // Run until the citizen has gone to bed and returned to work once.
        while flips < 2 && ticks < 10_000 {
            let recovering = resting;
            fatigue = fatigue_step(fatigue, recovering);
            let next = update_resting(resting, fatigue);
            if next != resting {
                flips += 1;
            }
            resting = next;
            ticks += 1;
        }
        assert_eq!(flips, 2, "expected exactly one rest and one return to work");
        assert!(ticks < 10_000, "cycle never completed");
    }

    #[test]
    fn shelter_slows_warmth_loss_in_the_cold() {
        let exposed = warmth_step(50.0, AMBIENT, false);
        let sheltered = warmth_step(50.0, AMBIENT, true);
        assert!(exposed < 50.0);
        assert!(sheltered > exposed, "a home must blunt the cold");
        assert!(sheltered < 50.0, "a home is not a heat source");
    }

    #[test]
    fn warmth_gain_ignores_shelter_and_stays_capped() {
        assert_eq!(warmth_step(WARMTH_MAX, 10.0, false), WARMTH_MAX);
        assert_eq!(
            warmth_step(50.0, 10.0, true),
            warmth_step(50.0, 10.0, false)
        );
    }

    #[test]
    fn warming_flips_only_at_the_hysteresis_thresholds() {
        let mid = (WARMTH_LOW + WARMTH_HIGH) / 2.0;
        assert!(!update_warming(false, mid));
        assert!(update_warming(true, mid));
        assert!(update_warming(false, WARMTH_LOW - 1.0));
        assert!(!update_warming(true, WARMTH_HIGH));
    }

    #[test]
    fn house_sites_are_distinct_and_fit_on_the_map() {
        let sites: Vec<IVec2> = (0..40).filter_map(house_site).collect();
        assert_eq!(sites.len(), 40, "40 houses must fit in the buildable rings");
        for (i, a) in sites.iter().enumerate() {
            assert!(a.x >= 0 && a.x <= R * 2 && a.y >= 0 && a.y <= R * 2);
            for b in &sites[i + 1..] {
                assert_ne!(a, b, "two houses landed on the same cell");
            }
        }
    }

    #[test]
    fn house_sites_never_cover_the_generator_or_the_forest() {
        let forest = forest_sites();
        for site in (0..40).filter_map(house_site) {
            assert_ne!(site, CENTER);
            assert!(
                !forest.contains(&site),
                "a house was built on a forest cell"
            );
        }
    }

    #[test]
    fn house_sites_run_out_beyond_the_buildable_rings() {
        let last = (0..)
            .take(10_000)
            .take_while(|i| house_site(*i).is_some())
            .count();
        assert!(last > 0, "at least one site must be buildable");
        assert!(
            house_site(last).is_none(),
            "sites must end, not wrap around"
        );
    }

    #[test]
    fn free_home_fills_each_house_to_capacity_before_the_next() {
        let sites: Vec<IVec2> = (0..2).filter_map(house_site).collect();
        let mut homes: Vec<IVec2> = Vec::new();
        for _ in 0..HOUSE_CAPACITY {
            let home = free_home(&sites, &homes).expect("first house has room");
            assert_eq!(home, sites[0]);
            homes.push(home);
        }
        assert_eq!(free_home(&sites, &homes), Some(sites[1]));
    }

    #[test]
    fn free_home_returns_none_when_every_bed_is_taken() {
        let sites: Vec<IVec2> = (0..2).filter_map(house_site).collect();
        let homes: Vec<IVec2> = sites
            .iter()
            .flat_map(|s| std::iter::repeat_n(*s, HOUSE_CAPACITY))
            .collect();
        assert_eq!(free_home(&sites, &homes), None);
    }

    #[test]
    fn a_citizen_delivers_wood_before_going_to_bed() {
        assert_eq!(choose_duty(false, true, true), Duty::Deliver);
        assert_eq!(choose_duty(false, false, true), Duty::Rest);
        assert_eq!(choose_duty(false, false, false), Duty::Gather);
    }

    #[test]
    fn freezing_overrides_every_other_duty() {
        assert_eq!(choose_duty(true, true, true), Duty::WarmUp);
    }
}
