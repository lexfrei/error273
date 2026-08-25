use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

// Nested game clocks. How fast a tick arrives in real time is a separate knob
// in the app wiring; these four numbers define game time and nothing else.
pub const TICKS_PER_HOUR: u64 = 1;
pub const HOURS_PER_DAY: u64 = 24;
pub const DAYS_PER_SEASON: u64 = 30;
pub const SEASONS_PER_YEAR: u64 = 4;

pub const fn ticks_per_day() -> u64 {
    TICKS_PER_HOUR * HOURS_PER_DAY
}

pub const fn ticks_per_season() -> u64 {
    ticks_per_day() * DAYS_PER_SEASON
}

pub const fn ticks_per_year() -> u64 {
    ticks_per_season() * SEASONS_PER_YEAR
}

/// Simulation rates are written in game-hour or game-day terms and converted
/// here, so no rate constant is a bare per-tick number that silently rescales
/// when the length of a day changes.
pub const fn per_hour(rate: f32) -> f32 {
    rate / TICKS_PER_HOUR as f32
}

pub const fn per_day(rate: f32) -> f32 {
    rate / ticks_per_day() as f32
}

#[derive(Resource, Default)]
pub struct Tick(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    fn from_index(index: u64) -> Season {
        match index % SEASONS_PER_YEAR {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

/// The game date behind the tick counter. Hours read like a clock and start at
/// zero; days and years are calendar labels and start at one.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Calendar {
    pub hour: u64,
    pub day: u64,
    pub season: Season,
    pub year: u64,
}

impl Default for Calendar {
    fn default() -> Self {
        calendar_at(0)
    }
}

pub fn calendar_at(tick: u64) -> Calendar {
    Calendar {
        hour: tick / TICKS_PER_HOUR % HOURS_PER_DAY,
        day: tick / ticks_per_day() % DAYS_PER_SEASON + 1,
        season: Season::from_index(tick / ticks_per_season()),
        year: tick / ticks_per_year() + 1,
    }
}

pub const R: i32 = 18;
pub const CENTER: IVec2 = IVec2::new(R, R);
pub const AMBIENT: f32 = -30.0;
pub const GENERATOR_HEAT: f32 = 66.0;
pub const HEAT_FALLOFF: f32 = 3.0;
// Fuel stocked above this level cannot make the fire any hotter; below it the
// grate is banked low and the warm zone shrinks with the pile.
pub const FULL_BURN_FUEL: u32 = 20;
pub const BURN_EVERY: u64 = 4 * TICKS_PER_HOUR;
pub const CITIZENS: usize = 30;

// Harvest patches sit on the rim, out past the last buildable house ring.
pub const PATCH_RADIUS: i32 = R - 1;
pub const WOOD_CELLS: usize = 8;
pub const WOOD_PER_CELL: u32 = 40;
pub const FOOD_CELLS: usize = 4;
pub const FOOD_PER_CELL: u32 = 60;

pub const NEED_MAX: f32 = 100.0;
pub const NEED_COUNT: usize = 3;
pub const START_WARMTH: f32 = 80.0;
// A house is shelter, not a stove: it only slows the bleed of body heat.
pub const SHELTER_DRAIN_FACTOR: f32 = 0.7;
// One haul of game is a carcass, not a plate, so a single unit settles a
// citizen for days.
pub const FOOD_PER_MEAL: u32 = 1;

pub const HOUSE_CAPACITY: usize = 3;
pub const HOUSES_PER_RING: usize = 12;
pub const HOUSE_RING_START: i32 = 5;
pub const HOUSE_RING_STEP: i32 = 2;
// Keep a margin between the outermost house ring and the patches on the rim.
pub const HOUSE_MAX_RADIUS: i32 = R - 3;
pub const HOUSE_WOOD_COST: u32 = 12;
// Hysteresis on the colony's wood policy: only a comfortable stock is diverted
// to a building site, and a project is not abandoned the moment it dips.
pub const FUEL_SPARE_HIGH: u32 = 45;
pub const FUEL_SPARE_LOW: u32 = 25;

pub const BIRTH_EVERY: u64 = 8 * TICKS_PER_HOUR;
pub const BIRTH_FUEL_MIN: u32 = 40;
pub const BIRTH_FOOD_MIN: u32 = 20;
pub const GROWTH_SHARE: f32 = 0.6;
// What the colony aims to hold per citizen, and so which stockpile a hauler
// judges to be the shorter one.
pub const FUEL_PER_CITIZEN: f32 = 1.3;
pub const FOOD_PER_CITIZEN: f32 = 0.6;
// Every block of citizens the generator has to warm costs another log per cycle,
// so growth is paid for twice: once in timber, then forever in fuel.
pub const POP_PER_EXTRA_BURN: usize = 20;

/// What a citizen can be short of. Systems walk `NEEDS` rather than naming
/// these one at a time, so a fourth need costs one table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedKind {
    Warmth,
    Rest,
    Food,
}

pub const NEEDS: [NeedKind; NEED_COUNT] = [NeedKind::Warmth, NeedKind::Rest, NeedKind::Food];

/// How one need behaves. `low`/`high` are the hysteresis band: a citizen starts
/// acting on the need at `low` and stops at `high`.
#[derive(Debug, Clone, Copy)]
pub struct NeedRules {
    pub decay: f32,
    pub recovery: f32,
    pub low: f32,
    pub high: f32,
    /// Level at or above which the need counts as comfortably met.
    pub comfort: f32,
    pub fatal: bool,
}

impl NeedKind {
    pub fn rules(self) -> NeedRules {
        match self {
            NeedKind::Warmth => NeedRules {
                decay: per_hour(1.0),
                recovery: per_hour(2.0),
                low: 25.0,
                high: 75.0,
                comfort: 60.0,
                fatal: true,
            },
            NeedKind::Rest => NeedRules {
                decay: per_day(24.0),
                recovery: per_hour(4.0),
                low: 40.0,
                high: 90.0,
                comfort: 41.0,
                fatal: false,
            },
            NeedKind::Food => NeedRules {
                decay: per_day(7.0),
                recovery: per_hour(60.0),
                low: 30.0,
                high: 90.0,
                comfort: 40.0,
                fatal: true,
            },
        }
    }
}

/// One need's state. `level` runs from 0 (desperate) to `NEED_MAX` (satisfied)
/// for every kind, so needs of different kinds compare directly.
#[derive(Debug, Clone, Copy)]
pub struct Need {
    pub level: f32,
    pub pressing: bool,
}

impl Need {
    /// Urgency on a common 0..=1 scale.
    pub fn pressure(self) -> f32 {
        (NEED_MAX - self.level) / NEED_MAX
    }
}

pub fn need_step(need: Need, kind: NeedKind, met: bool, decay_scale: f32) -> Need {
    let rules = kind.rules();
    let level = if met {
        (need.level + rules.recovery).min(NEED_MAX)
    } else {
        (need.level - rules.decay * decay_scale).max(0.0)
    };
    let pressing = if level <= rules.low {
        true
    } else if level >= rules.high {
        false
    } else {
        need.pressing
    };
    Need { level, pressing }
}

#[derive(Debug, Clone, Copy)]
pub struct Needs([Need; NEED_COUNT]);

impl Needs {
    /// A citizen who has just arrived: fed and rested, but out in the cold.
    pub fn newcomer() -> Self {
        let mut needs = [Need {
            level: NEED_MAX,
            pressing: false,
        }; NEED_COUNT];
        needs[NeedKind::Warmth as usize].level = START_WARMTH;
        Needs(needs)
    }

    pub fn get(&self, kind: NeedKind) -> Need {
        self.0[kind as usize]
    }

    pub fn level(&self, kind: NeedKind) -> f32 {
        self.get(kind).level
    }

    pub fn step(&mut self, kind: NeedKind, met: bool, decay_scale: f32) {
        self.0[kind as usize] = need_step(self.get(kind), kind, met, decay_scale);
    }

    /// True once a need that can kill has bottomed out.
    pub fn spent(&self) -> bool {
        NEEDS
            .into_iter()
            .any(|kind| kind.rules().fatal && self.level(kind) <= 0.0)
    }

    pub fn comfortable(&self, kind: NeedKind) -> bool {
        self.level(kind) >= kind.rules().comfort
    }

    /// Pressing needs, worst first. The sort is stable, so equal pressure keeps
    /// `NEEDS` order and the same colony state always decides the same way.
    pub fn pressing_by_urgency(&self) -> Vec<NeedKind> {
        let mut pressing: Vec<NeedKind> = NEEDS
            .into_iter()
            .filter(|kind| self.get(*kind).pressing)
            .collect();
        pressing.sort_by(|a, b| self.get(*b).pressure().total_cmp(&self.get(*a).pressure()));
        pressing
    }
}

/// What a citizen hauls, and what a patch yields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cargo {
    Wood,
    Food,
}

#[derive(Resource, Default)]
pub struct Generator {
    pub fuel: u32,
}

/// Everything the colony has put by, gathered into one borrow so readers do not
/// have to name each store separately.
#[derive(SystemParam)]
pub struct Stores<'w> {
    pub generator: Res<'w, Generator>,
    pub granary: Res<'w, Granary>,
    pub patches: Res<'w, Patches>,
}

#[derive(Resource, Default)]
pub struct Granary {
    pub food: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Patch {
    pub pos: IVec2,
    pub kind: Cargo,
    pub amount: u32,
}

#[derive(Resource)]
pub struct Patches(pub Vec<Patch>);

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
    pub needs: Needs,
    pub home: IVec2,
    pub carrying: Option<Cargo>,
    /// What this trip is for, settled once per trip so nobody turns around
    /// halfway when a stockpile ticks over.
    pub hauling: Cargo,
}

/// What a citizen spends this tick on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duty {
    WarmUp,
    Eat,
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

/// The most urgent thing a citizen could be doing. A need that kills outranks
/// the load on their back; tiredness does not.
pub fn choose_duty(needs: &Needs, carrying: Option<Cargo>) -> Duty {
    for kind in needs.pressing_by_urgency() {
        match kind {
            NeedKind::Warmth => return Duty::WarmUp,
            NeedKind::Food => return Duty::Eat,
            NeedKind::Rest if carrying.is_none() => return Duty::Rest,
            NeedKind::Rest => {}
        }
    }
    if carrying.is_some() {
        Duty::Deliver
    } else {
        Duty::Gather
    }
}

/// Where a citizen walks this tick. `source` is the patch they are working, if
/// any is left, and `drop_off` is wherever their load is wanted.
pub fn duty_target(
    duty: Duty,
    output: f32,
    home: IVec2,
    drop_off: IVec2,
    source: Option<IVec2>,
) -> IVec2 {
    match duty {
        Duty::WarmUp => warmth_target(output, home),
        Duty::Eat => CENTER,
        Duty::Deliver => drop_off,
        Duty::Rest => home,
        // With the patches stripped there is no work left, only warmth to find.
        Duty::Gather => source.unwrap_or_else(|| warmth_target(output, home)),
    }
}

/// Which stockpile the colony is shorter on, measured against what it wants to
/// hold per citizen. Ties go to wood: the cold kills faster than hunger.
pub fn haul_choice(fuel: u32, food: u32, population: usize) -> Cargo {
    let share = |stock: u32, per_head: f32| stock as f32 / (population as f32 * per_head).max(1.0);
    if share(food, FOOD_PER_CITIZEN) < share(fuel, FUEL_PER_CITIZEN) {
        Cargo::Food
    } else {
        Cargo::Wood
    }
}

/// The nearest patch a citizen can work: their own kind if any still stands,
/// otherwise whatever is left, so nobody idles beside a full hunting ground.
pub fn gather_source(patches: &Patches, want: Cargo, from: IVec2) -> Option<(IVec2, Cargo)> {
    let nearest = |kind: Cargo| {
        patches
            .0
            .iter()
            .filter(|patch| patch.kind == kind && patch.amount > 0)
            .min_by_key(|patch| (patch.pos - from).abs().max_element())
            .map(|patch| (patch.pos, patch.kind))
    };
    let other = match want {
        Cargo::Wood => Cargo::Food,
        Cargo::Food => Cargo::Wood,
    };
    nearest(want).or_else(|| nearest(other))
}

pub fn take_from_patch(patches: &mut Patches, pos: IVec2) {
    if let Some(patch) = patches.0.iter_mut().find(|patch| patch.pos == pos) {
        patch.amount = patch.amount.saturating_sub(1);
    }
}

pub fn patch_sites() -> Vec<Patch> {
    let ring = |count: usize, offset: f32, kind: Cargo, amount: u32| {
        (0..count)
            .map(|i| Patch {
                pos: ring_pos(
                    PATCH_RADIUS as f32,
                    (i as f32 + offset) / count as f32 * std::f32::consts::TAU,
                ),
                kind,
                amount,
            })
            .collect::<Vec<Patch>>()
    };
    let mut patches = ring(WOOD_CELLS, 0.0, Cargo::Wood, WOOD_PER_CELL);
    // Quarter-slot offset keeps the hunting grounds off the treelines.
    patches.extend(ring(FOOD_CELLS, 0.25, Cargo::Food, FOOD_PER_CELL));
    patches
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

/// First house with a free bed, given where everyone currently lives.
pub fn free_home(sites: &[IVec2], homes: &[IVec2]) -> Option<IVec2> {
    sites
        .iter()
        .find(|site| homes.iter().filter(|home| *home == *site).count() < HOUSE_CAPACITY)
        .copied()
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

/// Where a load is wanted. Only timber is ever wanted anywhere but the centre.
pub fn delivery_target(cargo: Cargo, diverting: bool, site: Option<IVec2>) -> IVec2 {
    match (cargo, site) {
        (Cargo::Wood, Some(pos)) if diverting => pos,
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

/// Share of the colony that has each need comfortably met.
pub fn met_shares(people: &[Needs]) -> [f32; NEED_COUNT] {
    let mut shares = [0.0; NEED_COUNT];
    if people.is_empty() {
        return shares;
    }
    let total = people.len() as f32;
    for kind in NEEDS {
        let met = people
            .iter()
            .filter(|needs| needs.comfortable(kind))
            .count();
        shares[kind as usize] = met as f32 / total;
    }
    shares
}

pub fn colony_thrives(shares: [f32; NEED_COUNT], fuel: u32, food: u32) -> bool {
    shares.iter().all(|share| *share >= GROWTH_SHARE)
        && fuel >= BIRTH_FUEL_MIN
        && food >= BIRTH_FOOD_MIN
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
                needs: Needs::newcomer(),
                home,
                carrying: None,
                hauling: Cargo::Wood,
            },
        ));
    }

    commands.insert_resource(Patches(patch_sites()));
}

pub fn advance_tick(mut tick: ResMut<Tick>) {
    tick.0 += 1;
}

pub fn advance_calendar(tick: Res<Tick>, mut calendar: ResMut<Calendar>) {
    *calendar = calendar_at(tick.0);
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

/// A colony with every need met and both stockpiles up takes in a newcomer, but
/// only if a bed is standing empty for them.
pub fn colony_growth(
    mut commands: Commands,
    tick: Res<Tick>,
    generator: Res<Generator>,
    granary: Res<Granary>,
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
    let people: Vec<Needs> = citizens.iter().map(|citizen| citizen.needs).collect();
    if !colony_thrives(met_shares(&people), generator.fuel, granary.food) {
        return;
    }
    commands.spawn((
        Pos(CENTER),
        Citizen {
            needs: Needs::newcomer(),
            home,
            carrying: None,
            hauling: haul_choice(generator.fuel, granary.food, people.len()),
        },
    ));
}

pub fn citizen_ai(
    mut commands: Commands,
    mut generator: ResMut<Generator>,
    mut granary: ResMut<Granary>,
    mut patches: ResMut<Patches>,
    mut construction: ResMut<Construction>,
    mut citizens: Query<(Entity, &mut Pos, &mut Citizen)>,
) {
    // One reading for the whole tick, so a citizen's luck does not depend on the
    // order the deliveries happen to land in.
    let output = generator_output(generator.fuel);
    let site_pos = construction.site.as_ref().map(|site| site.pos);
    let population = citizens.iter().count();

    for (entity, mut pos, mut citizen) in &mut citizens {
        let at_home = pos.0 == citizen.home;
        let at_centre = (pos.0 - CENTER).abs().max_element() <= 1;
        let duty = choose_duty(&citizen.needs, citizen.carrying);

        // Eating is what makes the food need met this tick, so it happens before
        // the needs are stepped.
        let eating = duty == Duty::Eat && at_centre && granary.food >= FOOD_PER_MEAL;
        if eating {
            granary.food -= FOOD_PER_MEAL;
        }
        let met = [
            heat_at(pos.0, output) >= 0.0,
            duty == Duty::Rest && at_home,
            eating,
        ];
        for (index, kind) in NEEDS.into_iter().enumerate() {
            let scale = if kind == NeedKind::Warmth && at_home {
                SHELTER_DRAIN_FACTOR
            } else {
                1.0
            };
            citizen.needs.step(kind, met[index], scale);
        }
        if citizen.needs.spent() {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(cargo) = citizen.carrying {
            let drop_off = delivery_target(cargo, construction.diverting, site_pos);
            if (pos.0 - drop_off).abs().max_element() <= 1 {
                match (cargo, construction.site.as_mut()) {
                    (Cargo::Wood, Some(site))
                        if log_goes_to_site(drop_off, site.pos, site.delivered) =>
                    {
                        site.delivered += 1;
                    }
                    (Cargo::Wood, _) => generator.fuel += 1,
                    (Cargo::Food, _) => granary.food += 1,
                }
                citizen.carrying = None;
                citizen.hauling = haul_choice(generator.fuel, granary.food, population);
            }
        }

        let source = gather_source(&patches, citizen.hauling, pos.0);
        if duty == Duty::Gather
            && let Some((cell, kind)) = source
            && cell == pos.0
        {
            take_from_patch(&mut patches, cell);
            citizen.carrying = Some(kind);
        }

        // Handing a load over or picking one up flips this tick's duty; nothing
        // else about the citizen has changed since it was chosen.
        let duty = choose_duty(&citizen.needs, citizen.carrying);
        let drop_off = citizen.carrying.map_or(CENTER, |cargo| {
            delivery_target(cargo, construction.diverting, site_pos)
        });
        let target = duty_target(
            duty,
            output,
            citizen.home,
            drop_off,
            source.map(|(cell, _)| cell),
        );
        if pos.0 != target {
            pos.0 = step_toward(pos.0, target);
        }
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

    fn need_at(level: f32) -> Need {
        Need {
            level,
            pressing: false,
        }
    }

    fn set(needs: &mut Needs, kind: NeedKind, level: f32, pressing: bool) {
        needs.0[kind as usize] = Need { level, pressing };
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
    fn a_full_need_presses_for_nothing_and_an_empty_one_presses_hardest() {
        assert_eq!(need_at(NEED_MAX).pressure(), 0.0);
        assert_eq!(need_at(0.0).pressure(), 1.0);
        assert!(need_at(20.0).pressure() > need_at(80.0).pressure());
    }

    #[test]
    fn every_need_recovers_when_met_and_slips_when_it_is_not() {
        for kind in NEEDS {
            let half = need_at(NEED_MAX / 2.0);
            assert!(
                need_step(half, kind, true, 1.0).level > half.level,
                "{kind:?} must recover while it is being met"
            );
            assert!(
                need_step(half, kind, false, 1.0).level < half.level,
                "{kind:?} must slip while it is neglected"
            );
        }
    }

    #[test]
    fn no_need_runs_past_full_or_below_empty() {
        for kind in NEEDS {
            assert_eq!(
                need_step(need_at(NEED_MAX), kind, true, 1.0).level,
                NEED_MAX
            );
            assert_eq!(need_step(need_at(0.0), kind, false, 1.0).level, 0.0);
        }
    }

    #[test]
    fn shelter_slows_a_neglected_need_without_reversing_it() {
        let start = need_at(NEED_MAX / 2.0);
        let exposed = need_step(start, NeedKind::Warmth, false, 1.0);
        let sheltered = need_step(start, NeedKind::Warmth, false, SHELTER_DRAIN_FACTOR);
        assert!(
            sheltered.level > exposed.level,
            "a home must blunt the cold"
        );
        assert!(sheltered.level < start.level, "a home is not a heat source");
    }

    #[test]
    fn recovery_ignores_the_decay_scale() {
        let start = need_at(NEED_MAX / 2.0);
        assert_eq!(
            need_step(start, NeedKind::Warmth, true, SHELTER_DRAIN_FACTOR).level,
            need_step(start, NeedKind::Warmth, true, 1.0).level
        );
    }

    #[test]
    fn every_need_presses_only_at_its_own_thresholds() {
        for kind in NEEDS {
            let rules = kind.rules();
            let mid = (rules.low + rules.high) / 2.0;
            let calm = need_step(need_at(mid), kind, false, 1.0);
            assert!(
                !calm.pressing || calm.level <= rules.low,
                "{kind:?} must not start pressing while still inside its band"
            );
            let latched = Need {
                level: mid,
                pressing: true,
            };
            let tended = need_step(latched, kind, true, 1.0);
            assert!(
                tended.pressing || tended.level >= rules.high,
                "{kind:?} must not stop pressing while still inside its band"
            );
            assert!(need_step(need_at(rules.low), kind, false, 1.0).pressing);
            assert!(
                !need_step(
                    Need {
                        level: rules.high,
                        pressing: true
                    },
                    kind,
                    true,
                    1.0
                )
                .pressing
            );
        }
    }

    #[test]
    fn every_need_has_a_workable_hysteresis_band() {
        for kind in NEEDS {
            let rules = kind.rules();
            assert!(rules.low < rules.high, "{kind:?} band is inverted");
            assert!(rules.high <= NEED_MAX, "{kind:?} can never stop pressing");
            assert!(
                rules.decay > 0.0 && rules.recovery > 0.0,
                "{kind:?} is inert"
            );
            assert!(
                rules.comfort > 0.0 && rules.comfort <= NEED_MAX,
                "{kind:?} comfort mark is off the scale"
            );
        }
    }

    #[test]
    fn a_full_work_rest_cycle_switches_state_exactly_twice() {
        let mut need = need_at(NEED_MAX);
        let mut flips = 0;
        let mut ticks = 0;
        while flips < 2 && ticks < 10_000 {
            let next = need_step(need, NeedKind::Rest, need.pressing, 1.0);
            if next.pressing != need.pressing {
                flips += 1;
            }
            need = next;
            ticks += 1;
        }
        assert_eq!(flips, 2, "expected exactly one rest and one return to work");
        assert!(ticks < 10_000, "cycle never completed");
    }

    #[test]
    fn only_cold_and_hunger_can_kill() {
        assert!(NeedKind::Warmth.rules().fatal);
        assert!(NeedKind::Food.rules().fatal);
        assert!(
            !NeedKind::Rest.rules().fatal,
            "nobody dies of tiredness here"
        );
    }

    #[test]
    fn a_citizen_dies_only_when_a_fatal_need_bottoms_out() {
        let mut needs = Needs::newcomer();
        set(&mut needs, NeedKind::Rest, 0.0, true);
        assert!(!needs.spent(), "exhaustion alone is survivable");
        set(&mut needs, NeedKind::Food, 0.0, true);
        assert!(needs.spent(), "starvation is not");
    }

    #[test]
    fn pressing_needs_come_out_worst_first() {
        let mut needs = Needs::newcomer();
        for kind in NEEDS {
            set(&mut needs, kind, kind.rules().low - 1.0, true);
        }
        set(&mut needs, NeedKind::Food, 1.0, true);
        let order = needs.pressing_by_urgency();
        assert_eq!(order.len(), NEED_COUNT);
        assert_eq!(order[0], NeedKind::Food, "the emptiest need leads");
    }

    #[test]
    fn a_calm_citizen_presses_for_nothing() {
        let needs = Needs::newcomer();
        assert!(
            needs.pressing_by_urgency().is_empty(),
            "a fresh citizen has no complaints"
        );
    }

    #[test]
    fn equal_pressure_breaks_toward_the_first_listed_need() {
        let mut needs = Needs::newcomer();
        for kind in [NeedKind::Rest, NeedKind::Food] {
            set(&mut needs, kind, 5.0, true);
        }
        assert_eq!(
            needs.pressing_by_urgency(),
            vec![NeedKind::Rest, NeedKind::Food]
        );
    }

    #[test]
    fn a_fatal_need_outranks_a_load_but_tiredness_does_not() {
        let mut needs = Needs::newcomer();
        set(&mut needs, NeedKind::Rest, NeedKind::Rest.rules().low, true);
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Wood)),
            Duty::Deliver,
            "wood is dropped off before bed"
        );
        assert_eq!(choose_duty(&needs, None), Duty::Rest);

        set(&mut needs, NeedKind::Food, 1.0, true);
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Wood)),
            Duty::Eat,
            "a starving citizen puts the load down"
        );
    }

    #[test]
    fn a_citizen_with_nothing_pressing_goes_to_work() {
        let needs = Needs::newcomer();
        assert_eq!(choose_duty(&needs, None), Duty::Gather);
        assert_eq!(choose_duty(&needs, Some(Cargo::Food)), Duty::Deliver);
    }

    #[test]
    fn each_duty_walks_to_its_own_destination() {
        let home = IVec2::new(3, 4);
        let drop_off = IVec2::new(7, 8);
        let patch = IVec2::new(1, 1);
        let lit = generator_output(FULL_BURN_FUEL);
        assert_eq!(
            duty_target(Duty::WarmUp, lit, home, drop_off, Some(patch)),
            CENTER
        );
        assert_eq!(
            duty_target(Duty::Eat, lit, home, drop_off, Some(patch)),
            CENTER
        );
        assert_eq!(
            duty_target(Duty::Deliver, lit, home, drop_off, Some(patch)),
            drop_off
        );
        assert_eq!(
            duty_target(Duty::Rest, lit, home, drop_off, Some(patch)),
            home
        );
        assert_eq!(
            duty_target(Duty::Gather, lit, home, drop_off, Some(patch)),
            patch
        );
    }

    #[test]
    fn a_gatherer_with_nothing_left_to_harvest_goes_looking_for_warmth() {
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
    fn haulers_follow_whichever_stockpile_is_shorter() {
        assert_eq!(haul_choice(200, 1, 30), Cargo::Food);
        assert_eq!(haul_choice(1, 200, 30), Cargo::Wood);
    }

    #[test]
    fn an_equally_stocked_colony_hauls_wood() {
        let pop = 30;
        let fuel = (pop as f32 * FUEL_PER_CITIZEN) as u32;
        let food = (pop as f32 * FOOD_PER_CITIZEN) as u32;
        assert_eq!(
            haul_choice(fuel, food, pop),
            Cargo::Wood,
            "ties go to the fire, which kills faster than hunger"
        );
    }

    #[test]
    fn haul_choice_survives_an_empty_colony() {
        assert_eq!(haul_choice(0, 0, 0), Cargo::Wood);
    }

    #[test]
    fn a_hauler_switches_kind_when_its_own_patches_are_stripped() {
        let mut patches = Patches(vec![
            Patch {
                pos: CENTER + IVec2::new(4, 0),
                kind: Cargo::Wood,
                amount: 0,
            },
            Patch {
                pos: CENTER + IVec2::new(0, 4),
                kind: Cargo::Food,
                amount: 5,
            },
        ]);
        let from = CENTER;
        assert_eq!(
            gather_source(&patches, Cargo::Wood, from),
            Some((CENTER + IVec2::new(0, 4), Cargo::Food)),
            "a stripped forest sends the hauler after game instead"
        );
        patches.0[1].amount = 0;
        assert_eq!(gather_source(&patches, Cargo::Wood, from), None);
    }

    #[test]
    fn a_hauler_prefers_its_own_kind_when_both_are_standing() {
        let patches = Patches(vec![
            Patch {
                pos: CENTER + IVec2::new(9, 0),
                kind: Cargo::Wood,
                amount: 5,
            },
            Patch {
                pos: CENTER + IVec2::new(0, 2),
                kind: Cargo::Food,
                amount: 5,
            },
        ]);
        assert_eq!(
            gather_source(&patches, Cargo::Wood, CENTER),
            Some((CENTER + IVec2::new(9, 0), Cargo::Wood)),
            "the nearer patch of the wrong kind does not win"
        );
    }

    #[test]
    fn taking_from_a_patch_draws_down_that_one_cell() {
        let pos = CENTER + IVec2::new(4, 0);
        let mut patches = Patches(vec![Patch {
            pos,
            kind: Cargo::Wood,
            amount: 2,
        }]);
        take_from_patch(&mut patches, pos);
        assert_eq!(patches.0[0].amount, 1);
        take_from_patch(&mut patches, pos);
        take_from_patch(&mut patches, pos);
        assert_eq!(
            patches.0[0].amount, 0,
            "a stripped patch must not underflow"
        );
    }

    #[test]
    fn food_goes_to_the_granary_even_while_a_house_is_going_up() {
        let site = IVec2::new(1, 2);
        assert_eq!(delivery_target(Cargo::Wood, true, Some(site)), site);
        assert_eq!(delivery_target(Cargo::Wood, false, Some(site)), CENTER);
        assert_eq!(
            delivery_target(Cargo::Food, true, Some(site)),
            CENTER,
            "nobody builds a house out of venison"
        );
    }

    #[test]
    fn wood_goes_to_the_fire_when_nothing_is_being_built() {
        assert_eq!(delivery_target(Cargo::Wood, true, None), CENTER);
        assert_eq!(delivery_target(Cargo::Wood, false, None), CENTER);
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
    fn met_shares_measure_each_need_separately() {
        let comfy = Needs::newcomer();
        let mut starving = Needs::newcomer();
        set(&mut starving, NeedKind::Food, 0.0, true);
        let shares = met_shares(&[comfy, starving]);
        assert_eq!(shares[NeedKind::Warmth as usize], 1.0);
        assert_eq!(shares[NeedKind::Rest as usize], 1.0);
        assert_eq!(shares[NeedKind::Food as usize], 0.5);
    }

    #[test]
    fn an_empty_colony_has_no_comfort_and_does_not_grow() {
        let shares = met_shares(&[]);
        assert_eq!(shares, [0.0; NEED_COUNT]);
        assert!(!colony_thrives(shares, u32::MAX, u32::MAX));
    }

    #[test]
    fn growth_needs_every_need_met_and_both_stockpiles() {
        let full = [1.0; NEED_COUNT];
        assert!(colony_thrives(full, BIRTH_FUEL_MIN, BIRTH_FOOD_MIN));
        assert!(
            !colony_thrives(full, BIRTH_FUEL_MIN - 1, BIRTH_FOOD_MIN),
            "no wood"
        );
        assert!(
            !colony_thrives(full, BIRTH_FUEL_MIN, BIRTH_FOOD_MIN - 1),
            "no food"
        );
        for kind in NEEDS {
            let mut shares = full;
            shares[kind as usize] = GROWTH_SHARE - 0.1;
            assert!(
                !colony_thrives(shares, BIRTH_FUEL_MIN, BIRTH_FOOD_MIN),
                "a colony short on {kind:?} must not grow"
            );
        }
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
    fn house_sites_never_cover_the_generator_or_a_harvest_patch() {
        let patches: Vec<IVec2> = patch_sites().into_iter().map(|patch| patch.pos).collect();
        for site in every_house_site() {
            assert_ne!(site, CENTER);
            assert!(
                !patches.contains(&site),
                "a house was built on a harvest patch"
            );
        }
    }

    #[test]
    fn harvest_patches_are_distinct_and_carry_both_kinds() {
        let patches = patch_sites();
        assert!(patches.iter().any(|p| p.kind == Cargo::Wood));
        assert!(patches.iter().any(|p| p.kind == Cargo::Food));
        for (i, a) in patches.iter().enumerate() {
            for b in &patches[i + 1..] {
                assert_ne!(a.pos, b.pos, "two patches share a cell");
            }
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
    fn the_clock_starts_at_the_first_hour_of_the_first_day() {
        let start = calendar_at(0);
        assert_eq!(start.hour, 0);
        assert_eq!(start.day, 1, "calendars have no day zero");
        assert_eq!(start.season, Season::Spring);
        assert_eq!(start.year, 1, "calendars have no year zero");
    }

    #[test]
    fn nested_clocks_roll_over_into_one_another() {
        assert_eq!(ticks_per_day(), TICKS_PER_HOUR * HOURS_PER_DAY);
        assert_eq!(ticks_per_season(), ticks_per_day() * DAYS_PER_SEASON);
        assert_eq!(ticks_per_year(), ticks_per_season() * SEASONS_PER_YEAR);

        let next_day = calendar_at(ticks_per_day());
        assert_eq!(next_day.hour, 0);
        assert_eq!(next_day.day, 2);
        assert_eq!(next_day.season, Season::Spring);

        let next_season = calendar_at(ticks_per_season());
        assert_eq!(next_season.day, 1, "a season starts on its first day");
        assert_eq!(next_season.season, Season::Summer);
        assert_eq!(next_season.year, 1);

        let next_year = calendar_at(ticks_per_year());
        assert_eq!(next_year.season, Season::Spring);
        assert_eq!(next_year.year, 2);
    }

    #[test]
    fn the_last_hour_of_a_year_has_not_rolled_over_yet() {
        let last = calendar_at(ticks_per_year() - 1);
        assert_eq!(last.year, 1);
        assert_eq!(last.season, Season::Winter);
        assert_eq!(last.day, DAYS_PER_SEASON);
        assert_eq!(last.hour, HOURS_PER_DAY - 1);
    }

    #[test]
    fn the_calendar_never_reads_off_its_own_dials() {
        for tick in (0..ticks_per_year() * 3).step_by(7) {
            let now = calendar_at(tick);
            assert!(now.hour < HOURS_PER_DAY);
            assert!(now.day >= 1 && now.day <= DAYS_PER_SEASON);
            assert!(now.year >= 1);
        }
    }

    #[test]
    fn the_seasons_run_in_order_and_wrap() {
        let order = [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ];
        assert_eq!(order.len(), SEASONS_PER_YEAR as usize);
        for (i, expected) in order.iter().enumerate() {
            assert_eq!(calendar_at(i as u64 * ticks_per_season()).season, *expected);
        }
        assert_eq!(
            calendar_at(SEASONS_PER_YEAR * ticks_per_season()).season,
            Season::Spring
        );
    }

    #[test]
    fn rates_written_per_hour_and_per_day_agree_with_the_clock() {
        assert_eq!(per_hour(1.0) * TICKS_PER_HOUR as f32, 1.0);
        assert_eq!(per_day(24.0), per_hour(24.0 / HOURS_PER_DAY as f32));
        assert!(
            per_day(1.0) < per_hour(1.0),
            "a daily rate is the slower one"
        );
    }
}
