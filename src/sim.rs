use bevy::prelude::*;

pub const R: i32 = 18;
pub const CENTER: IVec2 = IVec2::new(R, R);
pub const AMBIENT: f32 = -30.0;
pub const GENERATOR_HEAT: f32 = 66.0;
pub const HEAT_FALLOFF: f32 = 3.0;
// Fuel stocked above this level cannot make the fire any hotter; below it the
// grate is banked low and the warm zone shrinks with the pile.
pub const FULL_BURN_FUEL: u32 = 20;
pub const BURN_EVERY: u64 = 4;
pub const CITIZENS: usize = 30;
pub const FOREST_CELLS: usize = 8;
pub const WOOD_PER_CELL: u32 = 40;
pub const WARMTH_MAX: f32 = 100.0;
pub const WARMTH_GAIN: f32 = 2.0;
pub const WARMTH_DRAIN: f32 = 1.0;
// A house is shelter, not a stove: it only slows the bleed of body heat.
pub const SHELTER_DRAIN_FACTOR: f32 = 0.7;
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
pub const HOUSE_WOOD_COST: u32 = 12;
// Hysteresis on the colony's wood policy: only a comfortable stock is diverted
// to a building site, and a project is not abandoned the moment it dips.
pub const FUEL_SPARE_HIGH: u32 = 45;
pub const FUEL_SPARE_LOW: u32 = 25;
pub const BIRTH_EVERY: u64 = 8;
pub const BIRTH_FUEL_MIN: u32 = 40;
pub const GROWTH_WARM_MARK: f32 = 60.0;
pub const GROWTH_WARM_SHARE: f32 = 0.7;
pub const GROWTH_RESTED_SHARE: f32 = 0.6;
// Every block of citizens the generator has to warm costs another log per cycle,
// so growth is paid for twice: once in timber, then forever in fuel.
pub const POP_PER_EXTRA_BURN: usize = 20;
pub const START_WARMTH: f32 = 80.0;

#[derive(Resource, Default)]
pub struct Tick(pub u64);

#[derive(Resource)]
pub struct Generator {
    pub fuel: u32,
}

#[derive(Resource)]
pub struct Forest(pub Vec<(IVec2, u32)>);

/// The colony builds one house at a time; wood carried here is wood not burned.
#[derive(Resource, Default)]
pub struct Construction {
    pub site: Option<Site>,
    /// Whether the colony currently spares wood for building instead of the fire.
    pub diverting: bool,
}

pub struct Site {
    pub pos: IVec2,
    pub delivered: u32,
}

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

pub fn generator_output(fuel: u32) -> f32 {
    (fuel as f32 / FULL_BURN_FUEL as f32).min(1.0) * GENERATOR_HEAT
}

pub fn heat_at(p: IVec2, output: f32) -> f32 {
    let d = p.as_vec2().distance(CENTER.as_vec2());
    (output - d * HEAT_FALLOFF).max(0.0) + AMBIENT
}

/// The nearest warmth worth walking to: the generator while it still heats the
/// square, otherwise the citizen's own roof.
pub fn warmth_target(output: f32, home: IVec2) -> IVec2 {
    if heat_at(CENTER, output) > 0.0 {
        CENTER
    } else {
        home
    }
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

/// Where a citizen walks this tick. `wood` is the nearest standing tree, if one
/// is left, and `drop_off` is wherever the colony currently wants its wood.
pub fn duty_target(
    duty: Duty,
    output: f32,
    home: IVec2,
    drop_off: IVec2,
    wood: Option<IVec2>,
) -> IVec2 {
    match duty {
        Duty::WarmUp => warmth_target(output, home),
        Duty::Deliver => drop_off,
        Duty::Rest => home,
        // With the forest gone there is no work left, only warmth to look for.
        Duty::Gather => wood.unwrap_or_else(|| warmth_target(output, home)),
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

pub fn burn_amount(population: usize) -> u32 {
    1 + (population / POP_PER_EXTRA_BURN) as u32
}

pub fn update_diverting(diverting: bool, fuel: u32) -> bool {
    if fuel >= FUEL_SPARE_HIGH {
        true
    } else if fuel < FUEL_SPARE_LOW {
        false
    } else {
        diverting
    }
}

pub fn delivery_target(diverting: bool, site: Option<IVec2>) -> IVec2 {
    match site {
        Some(pos) if diverting => pos,
        _ => CENTER,
    }
}

pub fn should_start_building(diverting: bool, free_bed: bool) -> bool {
    diverting && !free_bed
}

/// Whether a log carried to `drop_off` becomes part of the house rather than
/// fuel. Timber past what the house needs would be thrown away, so it burns.
pub fn log_goes_to_site(drop_off: IVec2, site_pos: IVec2, delivered: u32) -> bool {
    drop_off == site_pos && delivered < HOUSE_WOOD_COST
}

/// Share of the colony that is warm, and share that is not worn out.
pub fn comfort_shares(people: &[(f32, f32)]) -> (f32, f32) {
    if people.is_empty() {
        return (0.0, 0.0);
    }
    let total = people.len() as f32;
    let warm = people
        .iter()
        .filter(|(w, _)| *w >= GROWTH_WARM_MARK)
        .count() as f32;
    let rested = people.iter().filter(|(_, f)| *f < FATIGUE_HIGH).count() as f32;
    (warm / total, rested / total)
}

pub fn colony_thrives(warm_share: f32, rested_share: f32, fuel: u32) -> bool {
    warm_share >= GROWTH_WARM_SHARE && rested_share >= GROWTH_RESTED_SHARE && fuel >= BIRTH_FUEL_MIN
}

/// Lowest plot with no house on it. Houses only ever go up, so this is simply
/// the next slot in the ring order.
pub fn next_house_site(existing: &[IVec2]) -> Option<IVec2> {
    (0usize..)
        .map(house_site)
        .take_while(Option::is_some)
        .flatten()
        .find(|site| !existing.contains(site))
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
                warmth: START_WARMTH,
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

pub fn burn_fuel(tick: Res<Tick>, mut generator: ResMut<Generator>, citizens: Query<&Citizen>) {
    if tick.0.is_multiple_of(BURN_EVERY) {
        let burned = burn_amount(citizens.iter().count());
        generator.fuel = generator.fuel.saturating_sub(burned);
    }
}

/// Finishes the house in progress, or opens a new plot once the beds are full.
pub fn construction(
    mut commands: Commands,
    mut construction: ResMut<Construction>,
    generator: Res<Generator>,
    houses: Query<&Pos, With<House>>,
    citizens: Query<&Citizen>,
) {
    construction.diverting = update_diverting(construction.diverting, generator.fuel);

    if let Some(site) = &construction.site {
        if site.delivered >= HOUSE_WOOD_COST {
            commands.spawn((Pos(site.pos), House));
            construction.site = None;
        }
        return;
    }

    let sites: Vec<IVec2> = houses.iter().map(|pos| pos.0).collect();
    let homes: Vec<IVec2> = citizens.iter().map(|citizen| citizen.home).collect();
    let free_bed = free_home(&sites, &homes).is_some();
    if !should_start_building(construction.diverting, free_bed) {
        return;
    }
    if let Some(pos) = next_house_site(&sites) {
        construction.site = Some(Site { pos, delivered: 0 });
    }
}

/// A warm, rested colony with fuel to spare takes in a newcomer, but only if a
/// bed is standing empty for them.
pub fn colony_growth(
    mut commands: Commands,
    tick: Res<Tick>,
    generator: Res<Generator>,
    houses: Query<&Pos, With<House>>,
    citizens: Query<&Citizen>,
) {
    if !tick.0.is_multiple_of(BIRTH_EVERY) {
        return;
    }
    let sites: Vec<IVec2> = houses.iter().map(|pos| pos.0).collect();
    let homes: Vec<IVec2> = citizens.iter().map(|citizen| citizen.home).collect();
    let Some(home) = free_home(&sites, &homes) else {
        return;
    };
    let people: Vec<(f32, f32)> = citizens
        .iter()
        .map(|citizen| (citizen.warmth, citizen.fatigue))
        .collect();
    let (warm_share, rested_share) = comfort_shares(&people);
    if !colony_thrives(warm_share, rested_share, generator.fuel) {
        return;
    }
    commands.spawn((
        Pos(CENTER),
        Citizen {
            warmth: START_WARMTH,
            fatigue: 0.0,
            home,
            carrying: false,
            warming: false,
            resting: false,
        },
    ));
}

pub fn citizen_ai(
    mut commands: Commands,
    mut generator: ResMut<Generator>,
    mut forest: ResMut<Forest>,
    mut construction: ResMut<Construction>,
    mut citizens: Query<(Entity, &mut Pos, &mut Citizen)>,
) {
    // One reading for the whole tick, so a citizen's luck does not depend on the
    // order the deliveries happen to land in.
    let output = generator_output(generator.fuel);
    let site_pos = construction.site.as_ref().map(|site| site.pos);
    let drop_off = delivery_target(construction.diverting, site_pos);
    for (entity, mut pos, mut citizen) in &mut citizens {
        let at_home = pos.0 == citizen.home;
        let heat = heat_at(pos.0, output);
        citizen.warmth = warmth_step(citizen.warmth, heat, at_home);
        if citizen.warmth <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        citizen.warming = update_warming(citizen.warming, citizen.warmth);

        if citizen.carrying && (pos.0 - drop_off).abs().max_element() <= 1 {
            match construction.site.as_mut() {
                Some(site) if log_goes_to_site(drop_off, site.pos, site.delivered) => {
                    site.delivered += 1;
                }
                _ => generator.fuel += 1,
            }
            citizen.carrying = false;
        }

        let duty = choose_duty(citizen.warming, citizen.carrying, citizen.resting);
        citizen.fatigue = fatigue_step(citizen.fatigue, duty == Duty::Rest && at_home);
        citizen.resting = update_resting(citizen.resting, citizen.fatigue);

        let wood = nearest_wood(&forest, pos.0);
        if duty == Duty::Gather && wood == Some(pos.0) {
            take_wood(&mut forest, pos.0);
            citizen.carrying = true;
        }
        // That pickup turns a gathering trip into a delivery run, so re-read the
        // duty before deciding where to walk.
        let target = duty_target(
            choose_duty(citizen.warming, citizen.carrying, citizen.resting),
            output,
            citizen.home,
            drop_off,
            wood,
        );
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

    /// Every plot the colony could ever build on, so invariants are checked
    /// across the whole set rather than the first ring or two.
    fn every_house_site() -> Vec<IVec2> {
        (0usize..)
            .map(house_site)
            .take_while(Option::is_some)
            .flatten()
            .collect()
    }

    #[test]
    fn heat_falls_off_with_distance() {
        let output = generator_output(FULL_BURN_FUEL);
        let near = heat_at(CENTER + IVec2::new(1, 0), output);
        let far = heat_at(CENTER + IVec2::new(10, 0), output);
        assert!(near > far);
        assert!(heat_at(CENTER, output) > 0.0);
    }

    #[test]
    fn heat_is_ambient_when_generator_is_off() {
        let dead = generator_output(0);
        assert_eq!(heat_at(CENTER, dead), AMBIENT);
        assert_eq!(heat_at(CENTER + IVec2::new(5, 5), dead), AMBIENT);
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
        let sites = every_house_site();
        assert!(sites.len() >= 40, "the colony needs room to grow into");
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
        for site in every_house_site() {
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

    #[test]
    fn a_bigger_city_burns_more_fuel_but_the_fire_never_stalls() {
        assert!(burn_amount(0) >= 1, "the generator always burns something");
        assert!(burn_amount(100) > burn_amount(10));
        assert!(
            burn_amount(10) <= burn_amount(11),
            "burn must not drop as the city grows"
        );
    }

    #[test]
    fn wood_reaches_the_building_site_only_while_the_colony_diverts_it() {
        let site = IVec2::new(1, 2);
        assert_eq!(delivery_target(true, Some(site)), site);
        assert_eq!(delivery_target(false, Some(site)), CENTER);
    }

    #[test]
    fn wood_goes_to_the_fire_when_nothing_is_being_built() {
        assert_eq!(delivery_target(true, None), CENTER);
        assert_eq!(delivery_target(false, None), CENTER);
    }

    #[test]
    fn diverting_wood_flips_only_at_the_hysteresis_thresholds() {
        let mid = (FUEL_SPARE_LOW + FUEL_SPARE_HIGH) / 2;
        assert!(
            !update_diverting(false, mid),
            "a middling stock does not start a project"
        );
        assert!(
            update_diverting(true, mid),
            "a started project is not abandoned mid-stock"
        );
        assert!(update_diverting(false, FUEL_SPARE_HIGH));
        assert!(!update_diverting(true, FUEL_SPARE_LOW - 1));
    }

    #[test]
    fn comfort_shares_measure_warmth_and_rest_separately() {
        let people = [
            (GROWTH_WARM_MARK + 1.0, 0.0),
            (GROWTH_WARM_MARK + 1.0, FATIGUE_HIGH),
            (GROWTH_WARM_MARK - 1.0, 0.0),
            (GROWTH_WARM_MARK - 1.0, FATIGUE_HIGH),
        ];
        let (warm, rested) = comfort_shares(&people);
        assert_eq!(warm, 0.5);
        assert_eq!(rested, 0.5);
    }

    #[test]
    fn an_empty_colony_has_no_comfort_and_does_not_grow() {
        let (warm, rested) = comfort_shares(&[]);
        assert_eq!(warm, 0.0);
        assert_eq!(rested, 0.0);
        assert!(!colony_thrives(warm, rested, u32::MAX));
    }

    #[test]
    fn growth_needs_warmth_rest_and_a_fuel_stock_together() {
        assert!(colony_thrives(1.0, 1.0, BIRTH_FUEL_MIN));
        assert!(
            !colony_thrives(GROWTH_WARM_SHARE - 0.1, 1.0, BIRTH_FUEL_MIN),
            "cold city"
        );
        assert!(
            !colony_thrives(1.0, GROWTH_RESTED_SHARE - 0.1, BIRTH_FUEL_MIN),
            "worn-out city"
        );
        assert!(
            !colony_thrives(1.0, 1.0, BIRTH_FUEL_MIN - 1),
            "no fuel to spare"
        );
    }

    #[test]
    fn building_starts_only_when_the_beds_are_full_and_wood_is_spare() {
        assert!(should_start_building(true, false));
        assert!(
            !should_start_building(true, true),
            "no point building on empty beds"
        );
        assert!(!should_start_building(false, false), "no wood to spare");
    }

    #[test]
    fn the_next_building_plot_skips_the_houses_already_standing() {
        let first = house_site(0).expect("site 0 exists");
        let second = house_site(1).expect("site 1 exists");
        assert_eq!(next_house_site(&[]), Some(first));
        assert_eq!(next_house_site(&[first]), Some(second));
        assert_eq!(
            next_house_site(&[second]),
            Some(first),
            "gaps get filled first"
        );
    }

    #[test]
    fn the_next_building_plot_runs_out_once_the_rings_are_built_out() {
        let all = every_house_site();
        assert!(!all.is_empty());
        assert_eq!(next_house_site(&all), None);
    }

    #[test]
    fn a_starved_generator_puts_out_less_heat() {
        assert_eq!(generator_output(0), 0.0);
        assert!(generator_output(FULL_BURN_FUEL / 2) < generator_output(FULL_BURN_FUEL));
        assert_eq!(generator_output(FULL_BURN_FUEL), GENERATOR_HEAT);
        assert_eq!(
            generator_output(FULL_BURN_FUEL * 10),
            GENERATOR_HEAT,
            "a full stock cannot be burned faster than the grate allows"
        );
    }

    #[test]
    fn the_warm_zone_shrinks_as_the_stock_runs_down() {
        let warm_radius = |fuel| {
            let output = generator_output(fuel);
            (0..=R)
                .filter(|d| heat_at(CENTER + IVec2::new(*d, 0), output) > 0.0)
                .count()
        };
        assert!(warm_radius(FULL_BURN_FUEL) > warm_radius(FULL_BURN_FUEL / 2));
        assert!(warm_radius(FULL_BURN_FUEL / 2) > warm_radius(0));
        assert_eq!(warm_radius(0), 0, "a dead generator warms nothing");
    }

    #[test]
    fn citizens_fall_back_to_their_own_roof_when_the_square_goes_cold() {
        let home = IVec2::new(3, 4);
        assert_eq!(
            warmth_target(generator_output(FULL_BURN_FUEL), home),
            CENTER
        );
        assert_eq!(warmth_target(generator_output(0), home), home);
    }

    #[test]
    fn each_duty_walks_to_its_own_destination() {
        let home = IVec2::new(3, 4);
        let drop_off = IVec2::new(7, 8);
        let tree = IVec2::new(1, 1);
        let lit = generator_output(FULL_BURN_FUEL);
        assert_eq!(
            duty_target(Duty::WarmUp, lit, home, drop_off, Some(tree)),
            CENTER
        );
        assert_eq!(
            duty_target(Duty::Deliver, lit, home, drop_off, Some(tree)),
            drop_off
        );
        assert_eq!(
            duty_target(Duty::Rest, lit, home, drop_off, Some(tree)),
            home
        );
        assert_eq!(
            duty_target(Duty::Gather, lit, home, drop_off, Some(tree)),
            tree
        );
    }

    #[test]
    fn a_gatherer_with_no_trees_left_goes_looking_for_warmth() {
        let home = IVec2::new(3, 4);
        let drop_off = IVec2::new(7, 8);
        assert_eq!(
            duty_target(
                Duty::Gather,
                generator_output(FULL_BURN_FUEL),
                home,
                drop_off,
                None
            ),
            CENTER,
            "while the fire burns, idle citizens huddle around it"
        );
        assert_eq!(
            duty_target(Duty::Gather, generator_output(0), home, drop_off, None),
            home,
            "once it is out, the only shelter left is their own roof"
        );
    }

    #[test]
    fn logs_past_what_the_house_needs_go_on_the_fire() {
        let site = IVec2::new(5, 5);
        assert!(log_goes_to_site(site, site, HOUSE_WOOD_COST - 1));
        assert!(
            !log_goes_to_site(site, site, HOUSE_WOOD_COST),
            "a finished house must not swallow timber"
        );
        assert!(
            !log_goes_to_site(CENTER, site, 0),
            "wood headed for the fire stays there"
        );
    }
}
