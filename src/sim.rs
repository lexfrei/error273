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

pub const fn days_per_year() -> u64 {
    DAYS_PER_SEASON * SEASONS_PER_YEAR
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

pub const fn per_year(rate: f32) -> f32 {
    rate / ticks_per_year() as f32
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
// The air over the year: a mean with a swing either side of it, mildest in the
// middle of summer and harshest in the middle of winter.
pub const AMBIENT_MEAN: f32 = -26.0;
pub const AMBIENT_SWING: f32 = 14.0;
// A colony gets one winter to learn on. The ramp is on the cold half only, so
// summers are the same every year.
pub const SEVERITY_FIRST: f32 = 0.55;
pub const SEVERITY_FULL_YEAR: u64 = 3;
// What a patch puts back on a growing day. Nothing grows in winter.
pub const WOOD_REGROWTH_PER_DAY: u32 = 6;
pub const FOOD_REGROWTH_PER_DAY: u32 = 3;
pub const GENERATOR_HEAT: f32 = 66.0;
pub const HEAT_FALLOFF: f32 = 3.0;
// Fuel stocked above this level cannot make the fire any hotter; below it the
// grate is banked low and the warm zone shrinks with the pile.
pub const FULL_BURN_FUEL: u32 = 20;
pub const BURN_EVERY: u64 = 4 * TICKS_PER_HOUR;
pub const CITIZENS: usize = 30;

// Harvest patches sit on the rim, out past the last buildable ring of plots.
pub const PATCH_RADIUS: i32 = R - 1;
pub const WOOD_CELLS: usize = 8;
pub const WOOD_PER_CELL: u32 = 80;
pub const FOOD_CELLS: usize = 4;
pub const FOOD_PER_CELL: u32 = 90;

// What the colony starts with. These belong beside the rates they are tuned
// against, not in the app wiring.
pub const START_FUEL: u32 = 20;
pub const START_FOOD: u32 = 60;
pub const START_RING: f32 = 2.0;

pub const NEED_MAX: f32 = 100.0;
pub const NEED_COUNT: usize = 3;
pub const START_WARMTH: f32 = 80.0;
// A house is shelter, not a stove: it only slows the bleed of body heat.
pub const SHELTER_DRAIN_FACTOR: f32 = 0.7;
// One haul of game is a carcass, not a plate, so a single unit settles a
// citizen for days.
pub const FOOD_PER_MEAL: u32 = 1;

pub const HOUSE_CAPACITY: usize = 3;
pub const PLOTS_PER_RING: usize = 12;
pub const PLOT_RING_START: i32 = 5;
pub const PLOT_RING_STEP: i32 = 2;
// Keep a margin between the outermost ring of plots and the patches on the rim.
pub const PLOT_MAX_RADIUS: i32 = R - 3;
pub const HOUSE_WOOD_COST: u32 = 12;
pub const HUT_WOOD_COST: u32 = 16;
pub const BOILER_WOOD_COST: u32 = 24;
// How much further a boiler pushes the generator's ceiling once the woodpile
// can keep it fed.
pub const UPGRADE_HEAT: f32 = 18.0;
pub const BUILDING_COUNT: usize = 3;
pub const CARGO_COUNT: usize = 2;
pub const SEASON_DAYS: usize = DAYS_PER_SEASON as usize;

// Stats are raised, not rolled. A childhood is banked hour by hour and settled
// at named ages, heaviest at the start, and what the colony could not account
// for survives as a residual rather than as the thing that decides anybody.
pub const STAT_COUNT: usize = 3;
pub const STAT_MIN: f32 = 0.05;
pub const STAT_MAX: f32 = 0.95;
/// A childhood exactly as good as the colony expects moves nothing.
pub const FORMATION_NEUTRAL: f32 = 0.5;
pub const MILESTONE_COUNT: usize = 4;
/// The ages a childhood settles at. Four named steps rather than one hidden
/// integral, so a child visibly becomes someone while the colony watches.
pub const MILESTONE_AGES: [f32; MILESTONE_COUNT] = [2.0, 6.0, 11.0, ADULT_AGE];
/// Heaviest on the first years, which is where the stunting window is.
pub const MILESTONE_WEIGHTS: [f32; MILESTONE_COUNT] = [0.60, 0.35, 0.20, 0.10];
// Checked where it cannot be skipped: the last milestone is the day a child
// becomes a worker, and the last stage still counts for something.
const _: () = assert!(MILESTONE_AGES[MILESTONE_COUNT - 1] == ADULT_AGE);
const _: () = assert!(MILESTONE_WEIGHTS[MILESTONE_COUNT - 1] > 0.0);
/// How far past adulthood a body can still make some of a bad childhood back.
pub const CATCHUP_UNTIL: f32 = 20.0;
pub const CATCHUP_WEIGHT: f32 = 0.12;
/// How far either side of the middle the part nobody can account for reaches.
pub const RESIDUAL_SPREAD: f32 = 0.25;
pub const STAT_SALT: u64 = 0x51;
// How much more than it spends a colony must be able to fetch before it takes
// on another mouth. The rate it is judged on is a season's average, and the
// season that binds is the one that puts nothing back, so this margin is the
// correction for measuring on the average -- the standard trap in a
// carrying-capacity estimate -- rather than a fudge factor.
pub const WINTER_MARGIN: f32 = 0.25;

// Lifecycle. Nobody dies on a birthday: hardiness holds until frailty sets in,
// then falls away towards a span of their own, and a cold night rolls against
// whatever is left of it.
pub const FRAILTY_ONSET: f32 = 40.0;
pub const LIFESPAN_BASE: f32 = 55.0;
pub const LIFESPAN_SPREAD: f32 = 0.1;
pub const COLD_SNAP_LETHALITY: f32 = 0.02;
pub const ADULT_AGE: f32 = 15.0;
pub const FERTILE_UNTIL: f32 = 45.0;
pub const MATURATION_SLOW: f32 = 0.6;
pub const MATURATION_FAST: f32 = 1.6;
// Chance one couple has a child over a season, before spare housing scales it.
pub const BIRTH_SEASON_CHANCE: f32 = 0.9;
pub const FOUNDER_AGE_MIN: f32 = 16.0;
pub const FOUNDER_AGE_MAX: f32 = 52.0;
// Salts keep the rolls a citizen makes for different things independent.
pub const LIFESPAN_SALT: u64 = 0x11;
pub const COLD_SNAP_SALT: u64 = 0x22;
pub const BIRTH_SALT: u64 = 0x33;
// What the colony keeps behind the fire before it will break ground on a
// project, so building never comes straight out of the next few nights.
// What the colony wants to be holding per head before it will break ground,
// and the fact that it will not start anything in a season that grows nothing
// back.
pub const BUILD_RESERVE_SHARE: f32 = 1.5;
// Past a handful of huts the grounds are as well worked as they are going to
// be; more sheds do not dress a carcass any faster.
pub const USEFUL_HUTS: usize = 3;
// The share of a colony that is children when it merely replaces itself is the
// share of a life spent as one: a child holds a dependent's place for
// ADULT_AGE of a LIFESPAN_BASE life. INVARIANT: a cap below this guarantees
// extinction, however healthy the colony looks, because births can never keep
// up with deaths. The slack above it is the only room a colony has to grow.
pub const REPLACEMENT_DEPENDENT_SHARE: f32 = ADULT_AGE / LIFESPAN_BASE;
pub const DEPENDENT_SLACK: f32 = 1.2;
pub const MAX_DEPENDENT_SHARE: f32 = REPLACEMENT_DEPENDENT_SHARE * DEPENDENT_SLACK;
// Hysteresis on the colony's wood policy: only a comfortable stock is diverted
// to a building site, and a project is not abandoned the moment it dips.
pub const FUEL_SPARE_HIGH: u32 = 58;
pub const FUEL_SPARE_LOW: u32 = 30;

pub const BIRTH_EVERY: u64 = 8 * TICKS_PER_HOUR;
pub const GROWTH_SHARE: f32 = 0.6;
// What the colony aims to hold per citizen, and so which stockpile a hauler
// judges to be the shorter one.
pub const FUEL_PER_CITIZEN: f32 = 1.3;
pub const FOOD_PER_CITIZEN: f32 = 0.6;
// Deadband on the haul decision, counted in hauls rather than units: how many
// trips of slack the other store gets before a hauler changes what they fetch.
pub const HAUL_SWITCH_HAULS: f32 = 4.0;
// ...but never wider than this much of a store's target. A band wider than the
// range it damps stops being a deadband and becomes a latch, and a latched
// hauler will walk past a full hunting ground while the granary empties.
pub const HAUL_SWITCH_MAX: f32 = 0.35;
// Every block of citizens the generator has to warm costs another log per cycle,
// so growth is paid for twice: once in timber, then forever in fuel.
pub const POP_PER_EXTRA_BURN: usize = 20;

/// What a colony raises in a citizen. Hidden: the card prints a word, never a
/// number, and only once the colony has watched enough work to have an opinion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Strength,
    Wits,
    Hardiness,
}

pub const STATS: [Stat; STAT_COUNT] = [Stat::Strength, Stat::Wits, Stat::Hardiness];

/// Whether a stat can still be made up after adulthood. The physical deficit of
/// a hungry childhood partly recovers and the cognitive one does not, which is
/// the whole reason the colony has two different kinds of stat to raise.
pub fn catches_up(stat: Stat) -> bool {
    !matches!(stat, Stat::Wits)
}

/// What a stat is raised on: the body eats, hardiness is warmed, and wits take
/// whatever the household had of either.
pub fn provision_for(stat: Stat, warmth: f32, food: f32) -> f32 {
    match stat {
        Stat::Strength => food,
        Stat::Hardiness => warmth,
        Stat::Wits => (warmth + food) / 2.0,
    }
}

/// What one stage of a childhood does to a stat. Multiplicative on purpose: the
/// same good year is worth more to a child who already has something to build
/// on, and a bad start cannot be bought back at full price.
pub fn milestone_step(stock: f32, provision: f32, weight: f32) -> f32 {
    let gain = (provision - FORMATION_NEUTRAL) * weight;
    (stock * (1.0 + gain)).clamp(STAT_MIN, STAT_MAX)
}

/// The part of a citizen the colony cannot account for. The literature is clear
/// that this share is large and unsystematic, so it is a residual rather than a
/// hidden input anybody could aim at.
fn residual(seed: u64, stat: Stat) -> f32 {
    let roll = noise(seed, STAT_SALT.wrapping_add(stat as u64));
    (FORMATION_NEUTRAL + (roll - 0.5) * 2.0 * RESIDUAL_SPREAD).clamp(STAT_MIN, STAT_MAX)
}

#[derive(Debug, Clone, Copy)]
pub struct Stats([f32; STAT_COUNT]);

impl Stats {
    pub fn of(&self, stat: Stat) -> f32 {
        self.0[stat as usize]
    }

    /// Somebody who walked in already grown. The colony never saw their
    /// childhood, so the residual is the whole of what it has to go on -- which
    /// is the honest rule for the founding party and for anyone who arrives.
    pub fn migrant(seed: u64) -> Stats {
        let mut stats = [0.0; STAT_COUNT];
        for stat in STATS {
            stats[stat as usize] = residual(seed, stat);
        }
        Stats(stats)
    }
}

/// A childhood in progress: what the colony had to give, banked hour by hour
/// and settled at the milestone ages.
#[derive(Debug, Clone, Copy)]
pub struct Upbringing {
    stock: [f32; STAT_COUNT],
    warmth: f32,
    food: f32,
    hours: f32,
    settled: usize,
}

impl Upbringing {
    /// A newborn starts from the residual and is raised from there.
    pub fn born(seed: u64) -> Upbringing {
        Upbringing {
            stock: Stats::migrant(seed).0,
            warmth: 0.0,
            food: 0.0,
            hours: 0.0,
            settled: 0,
        }
    }

    /// Somebody who arrived grown: nothing left to settle.
    pub fn grown(seed: u64) -> Upbringing {
        Upbringing {
            settled: MILESTONE_COUNT,
            ..Upbringing::born(seed)
        }
    }

    pub fn stats(&self) -> Stats {
        Stats(self.stock)
    }

    /// One hour of childhood, banked.
    pub fn observe(&mut self, warmth: f32, food: f32) {
        self.warmth += warmth;
        self.food += food;
        self.hours += 1.0;
    }

    /// Settle every milestone the child has reached, on the given stage.
    pub fn resolve(&mut self, age: f32, warmth: f32, food: f32) {
        while self.settled < MILESTONE_COUNT && age >= MILESTONE_AGES[self.settled] {
            let weight = MILESTONE_WEIGHTS[self.settled];
            for stat in STATS {
                let provision = provision_for(stat, warmth, food);
                self.stock[stat as usize] =
                    milestone_step(self.stock[stat as usize], provision, weight);
            }
            self.settled += 1;
        }
    }

    /// Settle on what has actually been banked since the last milestone, and
    /// start the next stage's ledger clean.
    pub fn settle_due(&mut self, age: f32) {
        if self.settled >= MILESTONE_COUNT || age < MILESTONE_AGES[self.settled] {
            return;
        }
        let hours = self.hours.max(1.0);
        self.resolve(age, self.warmth / hours, self.food / hours);
        self.warmth = 0.0;
        self.food = 0.0;
        self.hours = 0.0;
    }

    /// What a good adolescence can still buy back, diminishing with every year
    /// past adulthood until it is gone.
    pub fn catch_up(&mut self, age: f32, warmth: f32, food: f32) {
        if age < ADULT_AGE || age >= CATCHUP_UNTIL {
            return;
        }
        let left = (CATCHUP_UNTIL - age) / (CATCHUP_UNTIL - ADULT_AGE);
        for stat in STATS.into_iter().filter(|stat| catches_up(*stat)) {
            let provision = provision_for(stat, warmth, food);
            self.stock[stat as usize] = milestone_step(
                self.stock[stat as usize],
                provision,
                CATCHUP_WEIGHT * left * per_year(1.0),
            );
        }
    }
}

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
                // Rest counts as met so long as a citizen is not actually spent:
                // working through the band is the normal state, not a shortage.
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
/// for every kind, but each kind lives in a different part of that range, so
/// two levels are only comparable through `Needs::shortfall`.
#[derive(Debug, Clone, Copy)]
pub struct Need {
    pub level: f32,
    pub pressing: bool,
    /// What this need has cost its citizen since the last ballot: the time spent
    /// already acting on it and still not relieved, summed. A need that a
    /// citizen answers by sleeping or eating costs nothing at all -- the point
    /// is not discomfort but control failing, and a need swinging through its
    /// own band on schedule is control working.
    pub burden: f32,
}

/// How far through its own tolerance band a need at `level` has fallen: zero at
/// the mark where a citizen stops acting on it, one where they start.
pub fn shortfall_of(kind: NeedKind, level: f32) -> f32 {
    let rules = kind.rules();
    (rules.high - level) / (rules.high - rules.low)
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
    Need {
        level,
        pressing,
        // Only the part past the point of acting counts: a citizen already on
        // their way to the fire or the granary is not being failed yet.
        burden: need.burden + (shortfall_of(kind, level) - 1.0).max(0.0),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Needs {
    needs: [Need; NEED_COUNT],
    /// Ticks a citizen spent getting warm instead of working, since the last
    /// ballot. A boiler answers a cost rather than a need: a hauler always makes
    /// it back to the fire, so what the cold takes from them is hours, and no
    /// measure of how unmet a need got can see an hour.
    detour: u32,
}

impl Needs {
    /// A citizen who has just arrived: fed and rested, but out in the cold.
    pub fn newcomer() -> Self {
        let mut needs = [Need {
            level: NEED_MAX,
            pressing: false,
            burden: 0.0,
        }; NEED_COUNT];
        needs[NeedKind::Warmth as usize].level = START_WARMTH;
        let mut needs = Needs { needs, detour: 0 };
        needs.forget_before_ballot();
        needs
    }

    /// A founder. Everyone starting equally fed means everyone crossing the
    /// hunger threshold in the same hour, which is an artefact of setup rather
    /// than anything true about the colony, so their hunger is staggered across
    /// the party.
    pub fn founder(index: usize, of: usize) -> Self {
        let mut needs = Needs::newcomer();
        let rules = NeedKind::Food.rules();
        let along = (index + 1) as f32 / of.max(1) as f32;
        needs.needs[NeedKind::Food as usize].level = rules.low + along * (NEED_MAX - rules.low);
        needs.forget_before_ballot();
        needs
    }

    pub fn get(&self, kind: NeedKind) -> Need {
        self.needs[kind as usize]
    }

    pub fn level(&self, kind: NeedKind) -> f32 {
        self.get(kind).level
    }

    pub fn step(&mut self, kind: NeedKind, met: bool, decay_scale: f32) {
        self.needs[kind as usize] = need_step(self.get(kind), kind, met, decay_scale);
    }

    /// True once a need that can kill has bottomed out.
    pub fn spent(&self) -> bool {
        NEEDS
            .into_iter()
            .any(|kind| kind.rules().fatal && self.level(kind) <= 0.0)
    }

    /// How far a need has fallen through the band its citizen tolerates: zero at
    /// the mark where they stop acting on it, one where they start, and past one
    /// when it is worse than that. Bands differ per need, so this is what makes
    /// hunger and cold comparable at all.
    pub fn shortfall(&self, kind: NeedKind) -> f32 {
        shortfall_of(kind, self.level(kind))
    }

    /// Draw a line under the season: from here the ballot remembers only what
    /// happens next, and the hours it counts start again from none.
    pub fn forget_before_ballot(&mut self) {
        for kind in NEEDS {
            self.needs[kind as usize].burden = 0.0;
        }
        self.detour = 0;
    }

    /// One tick of a citizen's ballot window, and whether it went on getting
    /// warm rather than on work.
    pub fn spend(&mut self, on_getting_warm: bool) {
        if on_getting_warm {
            self.detour = self.detour.saturating_add(1);
        }
    }

    /// What the walk back to the fire has cost, in the same currency as a need:
    /// one tick of it weighs what one tick at the point of acting on a need
    /// weighs, so the four entries on the ballot compare directly.
    pub fn detour_burden(&self) -> f32 {
        self.detour as f32
    }

    pub fn comfortable(&self, kind: NeedKind) -> bool {
        self.level(kind) >= kind.rules().comfort
    }

    /// Pressing needs, worst first. The sort is stable, so equally short needs
    /// keep `NEEDS` order and the same colony state always decides the same way.
    pub fn pressing_by_urgency(&self) -> Vec<NeedKind> {
        let mut pressing: Vec<NeedKind> = NEEDS
            .into_iter()
            .filter(|kind| self.get(*kind).pressing)
            .collect();
        pressing.sort_by(|a, b| self.shortfall(*b).total_cmp(&self.shortfall(*a)));
        pressing
    }
}

/// What a citizen hauls, and what a patch yields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cargo {
    Wood,
    Food,
}

impl Cargo {
    pub fn other(self) -> Cargo {
        match self {
            Cargo::Wood => Cargo::Food,
            Cargo::Food => Cargo::Wood,
        }
    }
}

#[derive(Resource, Default)]
pub struct Generator {
    pub fuel: u32,
}

/// What the colony decides with: the project in hand, the last ballot, and the
/// office's thumb on the scale.
#[derive(SystemParam)]
pub struct Council<'w> {
    pub construction: ResMut<'w, Construction>,
    pub ballot: ResMut<'w, Ballot>,
    pub mayor: Res<'w, Mayor>,
}

/// What the colony has been measuring about itself: what it holds, what it has
/// been fetching, and what it has put up.
#[derive(SystemParam)]
pub struct Ledger<'w> {
    pub trend: Res<'w, Trend>,
    pub flow: Res<'w, Flow>,
    pub built: Res<'w, Built>,
}

/// What the world outside is doing, for the systems that only read it.
#[derive(SystemParam)]
pub struct Outside<'w> {
    pub tick: Res<'w, Tick>,
    pub calendar: Res<'w, Calendar>,
    pub air: Res<'w, Air>,
}

/// Everything the colony's own work touches, gathered into one borrow so the
/// systems that change it keep a readable signature.
#[derive(SystemParam)]
pub struct Colony<'w> {
    pub generator: ResMut<'w, Generator>,
    pub granary: ResMut<'w, Granary>,
    pub patches: ResMut<'w, Patches>,
    pub construction: ResMut<'w, Construction>,
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
    /// What this patch grows back towards, and never past.
    pub cap: u32,
}

#[derive(Resource)]
pub struct Patches(pub Vec<Patch>);

/// The colony runs one project at a time; wood carried there is wood not burned.
#[derive(Resource, Default)]
pub struct Construction {
    pub site: Option<Site>,
    /// Whether the colony currently spares wood for building instead of the fire.
    pub diverting: bool,
}

pub struct Site {
    pub pos: IVec2,
    pub building: Building,
    pub delivered: u32,
}

#[derive(Component)]
pub struct Pos(pub IVec2);

/// What a citizen can put up. Systems walk `BUILDINGS` rather than naming each
/// kind, so a fourth building costs one table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Building {
    House,
    HuntersHut,
    GeneratorUpgrade,
}

pub const BUILDINGS: [Building; BUILDING_COUNT] = [
    Building::House,
    Building::HuntersHut,
    Building::GeneratorUpgrade,
];

#[derive(Debug, Clone, Copy)]
pub struct BuildingRules {
    pub cost: u32,
    pub name: &'static str,
}

impl Building {
    pub fn rules(self) -> BuildingRules {
        match self {
            Building::House => BuildingRules {
                cost: HOUSE_WOOD_COST,
                name: "House",
            },
            Building::HuntersHut => BuildingRules {
                cost: HUT_WOOD_COST,
                name: "Hut",
            },
            Building::GeneratorUpgrade => BuildingRules {
                cost: BOILER_WOOD_COST,
                name: "Boiler",
            },
        }
    }
}

/// Seeds handed out to citizens in the order they are born, so no two roll the
/// same numbers and a run replays exactly.
#[derive(Resource, Default)]
pub struct Lineage(pub u64);

impl Lineage {
    pub fn next(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}

/// The mayor's office: a weight per building type, added to the ballot before
/// it is read. Deliberately inert -- this is data with no behaviour, and the
/// place a policy layer from outside the colony reaches in. Anything that wants
/// to scale or gate these weights wraps the struct rather than editing voting.
#[derive(Resource, Default)]
pub struct Mayor {
    pub bias: [f32; BUILDING_COUNT],
}

/// The last ballot the colony held, kept so it can be shown.
#[derive(Resource, Default)]
pub struct Ballot {
    pub tally: [f32; BUILDING_COUNT],
}

/// A finished building standing on a plot.
#[derive(Component)]
pub struct Structure(pub Building);

/// How many of each building stand, recounted every tick so nothing has to be
/// kept in step by hand.
#[derive(Resource, Default)]
pub struct Built([usize; BUILDING_COUNT]);

impl Built {
    pub fn of(&self, building: Building) -> usize {
        self.0[building as usize]
    }
}

#[derive(Component)]
pub struct Citizen {
    pub needs: Needs,
    /// What the colony has raised in them, and what it started from.
    pub upbringing: Upbringing,
    /// Years lived, on the same clock the calendar prints.
    pub age: f32,
    /// The span this one would reach if nothing got them first.
    pub lifespan: f32,
    /// Fixed per citizen, so their rolls are their own and replay the same.
    pub seed: u64,
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

/// Deterministic noise in `[0, 1)` from a pair of integers. The simulation has
/// no entropy source and wants none: two runs of the same build must tell the
/// same story, or a balance log is worth nothing.
pub fn noise(seed: u64, salt: u64) -> f32 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

/// The span a citizen would reach if nothing else got them first, scattered
/// about the expected one so a founding party does not die together.
pub fn lifespan_of(seed: u64) -> f32 {
    let drift = (noise(seed, LIFESPAN_SALT) - 0.5) * 2.0 * LIFESPAN_SPREAD;
    LIFESPAN_BASE * (1.0 + drift)
}

/// How well a citizen still shrugs off a cold night. Whole while they are
/// young, falling away once frailty sets in, and gone by their own span. There
/// is no age at which anyone simply dies.
pub fn hardiness(age: f32, lifespan: f32) -> f32 {
    if age <= FRAILTY_ONSET {
        return 1.0;
    }
    let remaining = (lifespan - FRAILTY_ONSET).max(1.0);
    (1.0 - (age - FRAILTY_ONSET) / remaining).clamp(0.0, 1.0)
}

/// Whether a cold night carries a citizen off. What hardiness they have left is
/// what stands between them and the roll, so winter is what does the killing.
pub fn cold_takes(hardiness: f32, roll: f32) -> bool {
    roll < (1.0 - hardiness) * COLD_SNAP_LETHALITY
}

pub fn is_adult(age: f32) -> bool {
    age >= ADULT_AGE
}

/// How fast children grow up. A colony with warmth and food to spare raises
/// them faster, so the shape of its age pyramid is an output of how it is doing
/// rather than a number anyone set.
pub fn maturation_rate(warm_share: f32, fed_share: f32) -> f32 {
    let comfort = ((warm_share + fed_share) / 2.0).clamp(0.0, 1.0);
    MATURATION_SLOW + comfort * (MATURATION_FAST - MATURATION_SLOW)
}

/// Couples the colony could have, from those grown and still of an age for it.
pub fn couples(ages: &[f32]) -> usize {
    ages.iter()
        .filter(|age| is_adult(**age) && **age <= FERTILE_UNTIL)
        .count()
        / 2
}

/// The chance one couple has a child over a season. Spare beds scale it rather
/// than gating it: a colony with room grows, a full one tails off, and neither
/// arrives as the step function that makes a generation land all at once.
pub fn birth_chance_per_season(spare_beds: usize, couples: usize) -> f32 {
    if couples == 0 {
        return 0.0;
    }
    let room = (spare_beds as f32 / couples as f32).min(1.0);
    BIRTH_SEASON_CHANCE * room
}

/// The same chance spread over the checks a season is made of.
pub fn birth_chance_per_check(seasonal: f32) -> f32 {
    seasonal / (ticks_per_season() / BIRTH_EVERY) as f32
}

/// The age a founder starts at, spread across the party so that frailty, and
/// so death, does not arrive for all of them in the same winter.
pub fn founder_age(index: usize, of: usize) -> f32 {
    let along = index as f32 / (of.max(2) - 1) as f32;
    FOUNDER_AGE_MIN + along.min(1.0) * (FOUNDER_AGE_MAX - FOUNDER_AGE_MIN)
}

pub fn ring_pos(radius: f32, angle: f32) -> IVec2 {
    CENTER
        + IVec2::new(
            (radius * angle.cos()).round() as i32,
            (radius * angle.sin()).round() as i32,
        )
}

/// How hard the cold bites in a given year. The first winter is scaled back so
/// a colony gets one to learn on, reaching full depth by `SEVERITY_FULL_YEAR`.
pub fn severity(year: u64) -> f32 {
    let years_in = year.saturating_sub(1) as f32;
    let ramp = years_in / (SEVERITY_FULL_YEAR.saturating_sub(1).max(1)) as f32;
    (SEVERITY_FIRST + ramp * (1.0 - SEVERITY_FIRST)).min(1.0)
}

/// The air outside the generator's reach on the day a tick falls in. A cosine
/// through the year, so the drift from one day to the next is smooth and the
/// extremes land mid-season rather than on a boundary.
pub fn ambient_at(tick: u64) -> f32 {
    let day = tick / ticks_per_day() % days_per_year();
    let year = tick / ticks_per_year() + 1;
    let midsummer = days_per_year() / 4 + days_per_year() / 8;
    let phase = (day as f32 - midsummer as f32) / days_per_year() as f32 * std::f32::consts::TAU;
    let through_the_year = phase.cos();
    let swing = if through_the_year < 0.0 {
        AMBIENT_SWING * severity(year)
    } else {
        AMBIENT_SWING
    };
    AMBIENT_MEAN + swing * through_the_year
}

pub fn is_growing_season(season: Season) -> bool {
    !matches!(season, Season::Winter)
}

pub fn regrowth_per_day(kind: Cargo) -> u32 {
    match kind {
        Cargo::Wood => WOOD_REGROWTH_PER_DAY,
        Cargo::Food => FOOD_REGROWTH_PER_DAY,
    }
}

/// What a patch holds after a day of growing back, never past its own cap.
pub fn regrowth_step(amount: u32, cap: u32, kind: Cargo, growing: bool) -> u32 {
    if !growing {
        return amount;
    }
    (amount + regrowth_per_day(kind)).min(cap)
}

pub fn generator_output(fuel: u32, upgrades: usize) -> f32 {
    let ceiling = GENERATOR_HEAT + upgrades as f32 * UPGRADE_HEAT;
    (fuel as f32 / FULL_BURN_FUEL as f32).min(1.0) * ceiling
}

/// The air the colony is standing in: what the fire is putting out, and how
/// cold it is outside the fire's reach. The two always travel together.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Air {
    pub output: f32,
    pub ambient: f32,
}

impl Air {
    pub fn heat_at(self, p: IVec2) -> f32 {
        let d = p.as_vec2().distance(CENTER.as_vec2());
        (self.output - d * HEAT_FALLOFF).max(0.0) + self.ambient
    }

    /// The nearest warmth worth walking to: the generator while it still heats
    /// the square, otherwise the citizen's own roof.
    pub fn warmth_target(self, home: IVec2) -> IVec2 {
        if self.heat_at(CENTER) > 0.0 {
            CENTER
        } else {
            home
        }
    }
}

pub fn step_toward(from: IVec2, to: IVec2) -> IVec2 {
    from + (to - from).signum()
}

/// The most urgent thing a citizen could be doing. A need that kills outranks
/// the load on their back; tiredness does not.
pub fn choose_duty(needs: &Needs, carrying: Option<Cargo>, grown: bool) -> Duty {
    for kind in needs.pressing_by_urgency() {
        match kind {
            NeedKind::Warmth => return Duty::WarmUp,
            NeedKind::Food => return Duty::Eat,
            NeedKind::Rest if carrying.is_none() => return Duty::Rest,
            NeedKind::Rest => {}
        }
    }
    if !grown {
        // Children are mouths, not hands: they keep to the house until grown.
        return Duty::Rest;
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
    air: Air,
    home: IVec2,
    drop_off: IVec2,
    source: Option<IVec2>,
) -> IVec2 {
    match duty {
        Duty::WarmUp => air.warmth_target(home),
        Duty::Eat => CENTER,
        Duty::Deliver => drop_off,
        Duty::Rest => home,
        // With the patches stripped there is no work left, only warmth to find.
        Duty::Gather => source.unwrap_or_else(|| air.warmth_target(home)),
    }
}

/// How well stocked a store is against what the colony wants to hold per head.
/// One is the target; below one it is running short.
pub fn stock_share(stock: u32, per_head: f32, population: usize) -> f32 {
    stock as f32 / (population as f32 * per_head).max(1.0)
}

/// How far behind the other store has to fall before a hauler changes what they
/// fetch. Sized in hauls, not units: a hut makes each trip of game worth more,
/// so the same number of units is a bigger swing, and a band that did not widen
/// with it would have the workforce overshoot on every correction.
pub fn haul_switch_margin(population: usize, huts: usize) -> f32 {
    let per_haul = food_yield(huts) as f32 / (population as f32 * FOOD_PER_CITIZEN).max(1.0);
    (HAUL_SWITCH_HAULS * per_haul).min(HAUL_SWITCH_MAX)
}

/// What a store will hold once everyone already fetching that kind gets home.
pub fn projected_stock(stock: u32, inbound: usize, per_haul: u32) -> u32 {
    stock + inbound as u32 * per_haul
}

/// The colony as a hauler sees it when deciding what to fetch next.
#[derive(Debug, Clone, Copy)]
pub struct Supply {
    pub fuel: u32,
    pub food: u32,
    /// Haulers already committed to each kind, by `Cargo` order.
    pub inbound: [usize; CARGO_COUNT],
    pub population: usize,
    pub huts: usize,
}

impl Supply {
    /// How well stocked a store will be once what is already walking towards it
    /// arrives. Judging on the store alone sends the whole workforce after a
    /// shortage that the first few haulers have already covered, and a round
    /// trip is long enough that nobody finds out until it is too late.
    /// Whether a store is still short of what the colony wants to hold per head.
    pub fn wants(&self, cargo: Cargo) -> bool {
        self.projected_share(cargo) < 1.0
    }

    pub fn projected_share(&self, cargo: Cargo) -> f32 {
        let (stock, per_haul, per_head) = match cargo {
            Cargo::Wood => (self.fuel, 1, FUEL_PER_CITIZEN),
            Cargo::Food => (self.food, food_yield(self.huts), FOOD_PER_CITIZEN),
        };
        let inbound = self.inbound[cargo as usize];
        stock_share(
            projected_stock(stock, inbound, per_haul),
            per_head,
            self.population,
        )
    }
}

/// Which store a hauler works next. They keep to the kind they were on until
/// the other store has fallen a clear margin behind, so a stockpile crossing
/// its target does not swing the whole workforce round at once.
pub fn haul_choice(current: Cargo, supply: Supply) -> Cargo {
    let working = supply.projected_share(current);
    let other = supply.projected_share(current.other());
    if other + haul_switch_margin(supply.population, supply.huts) < working {
        current.other()
    } else {
        current
    }
}

/// The nearest patch a citizen can work: their own kind if any still stands,
/// otherwise whatever is left, but only if the colony actually wants it.
pub fn gather_source(
    patches: &Patches,
    want: Cargo,
    from: IVec2,
    take_other: bool,
) -> Option<(IVec2, Cargo)> {
    let nearest = |kind: Cargo| {
        patches
            .0
            .iter()
            .filter(|patch| patch.kind == kind && patch.amount > 0)
            .min_by_key(|patch| (patch.pos - from).abs().max_element())
            .map(|patch| (patch.pos, patch.kind))
    };
    match nearest(want) {
        found @ Some(_) => found,
        // Falling back on a store the colony already has more than enough of
        // buys nothing and keeps a citizen out in the cold to do it.
        None if take_other => nearest(want.other()),
        None => None,
    }
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
                cap: amount,
            })
            .collect::<Vec<Patch>>()
    };
    let mut patches = ring(WOOD_CELLS, 0.0, Cargo::Wood, WOOD_PER_CELL);
    // Quarter-slot offset keeps the hunting grounds off the treelines.
    patches.extend(ring(FOOD_CELLS, 0.25, Cargo::Food, FOOD_PER_CELL));
    patches
}

/// Fixed plot for the n-th building, filling one ring before moving outward so
/// that putting something up never displaces what already stands.
pub fn plot_site(index: usize) -> Option<IVec2> {
    let ring = index / PLOTS_PER_RING;
    let radius = PLOT_RING_START + ring as i32 * PLOT_RING_STEP;
    if radius > PLOT_MAX_RADIUS {
        return None;
    }
    let slot = index % PLOTS_PER_RING;
    // Offset every other ring by half a slot so streets are not fully radial.
    let offset = if ring.is_multiple_of(2) { 0.0 } else { 0.5 };
    let angle = (slot as f32 + offset) / PLOTS_PER_RING as f32 * std::f32::consts::TAU;
    Some(ring_pos(radius as f32, angle))
}

/// First house with a free bed, given where everyone currently lives.
pub fn free_home(sites: &[IVec2], homes: &[IVec2]) -> Option<IVec2> {
    sites
        .iter()
        .find(|site| homes.iter().filter(|home| *home == *site).count() < HOUSE_CAPACITY)
        .copied()
}

/// Beds standing empty across every house, which is what scales the birth rate.
pub fn spare_beds(sites: &[IVec2], homes: &[IVec2]) -> usize {
    (sites.len() * HOUSE_CAPACITY).saturating_sub(homes.len())
}

/// Lowest plot with nothing on it. Buildings only ever go up, so this is simply
/// the next slot in the ring order, shared by every kind.
pub fn next_plot(existing: &[IVec2]) -> Option<IVec2> {
    (0usize..)
        .map(plot_site)
        .take_while(Option::is_some)
        .flatten()
        .find(|site| !existing.contains(site))
}

pub fn burn_amount(population: usize, upgrades: usize) -> u32 {
    1 + (population / POP_PER_EXTRA_BURN) as u32 + upgrades as u32
}

/// What one haul of game puts in the granary. Every hut standing means the
/// carcass comes back dressed rather than dragged whole.
pub fn food_yield(huts: usize) -> u32 {
    1 + huts.min(USEFUL_HUTS) as u32
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

/// The building that answers a given need, and the whole of the mapping: every
/// need has exactly one remedy and every building is somebody's.
pub fn building_for(kind: NeedKind) -> Building {
    match kind {
        NeedKind::Warmth => Building::GeneratorUpgrade,
        NeedKind::Rest => Building::House,
        NeedKind::Food => Building::HuntersHut,
    }
}

/// How one citizen votes, in two tiers that are never compared. First
/// survival: a need they were already acting on and still not getting, summed
/// over the hours it stayed that way. If anything shows there, that is the
/// vote. Only a citizen the colony is holding up for gets to the second tier,
/// efficiency, and votes on what wastes the most of their day -- or abstains,
/// which is a legal thing for a contented citizen to do. The strict comparison
/// keeps the first of any equals, so ties resolve in table order.
pub fn vote_of(needs: &Needs) -> Option<Building> {
    let mut choice = NEEDS[0];
    for kind in NEEDS {
        if needs.get(kind).burden > needs.get(choice).burden {
            choice = kind;
        }
    }
    if needs.get(choice).burden > 0.0 {
        return Some(building_for(choice));
    }
    // Nothing went unanswered, so the colony is holding for this citizen and
    // the question is no longer survival but waste. The two are never compared:
    // a starving colony votes food whatever its haulers are walking, and a
    // comfortable one is free to spend the ballot on efficiency.
    if needs.detour_burden() > 0.0 {
        return Some(Building::GeneratorUpgrade);
    }
    None
}

/// The ballot: one voice each, on top of whatever the mayor's office is
/// leaning on. The mayor does not get a vote, only a thumb on the scale.
pub fn tally_votes(people: &[Needs], mayor: &Mayor) -> [f32; BUILDING_COUNT] {
    let mut tally = mayor.bias;
    for needs in people.iter().filter_map(vote_of) {
        tally[needs as usize] += 1.0;
    }
    tally
}

pub fn winner(tally: [f32; BUILDING_COUNT]) -> Building {
    let mut best = BUILDINGS[0];
    for building in BUILDINGS {
        if tally[building as usize] > tally[best as usize] {
            best = building;
        }
    }
    best
}

/// What the colony puts up next. The ballot decides, except that with every bed
/// taken a house is not a preference but the precondition for growing at all.
pub fn next_project(tally: [f32; BUILDING_COUNT], free_bed: bool) -> Building {
    if !free_bed {
        return Building::House;
    }
    winner(tally)
}

/// Whether a citizen takes a meal this tick. Hunger is settled wherever they
/// are already standing at the granary rather than only when eating is the most
/// pressing thing they could be doing, so nobody starves on a full store
/// because the cold happened to be worse.
pub fn takes_a_meal(needs: &Needs, at_granary: bool, food: u32) -> bool {
    needs.get(NeedKind::Food).pressing && at_granary && food >= FOOD_PER_MEAL
}

/// Whether the colony can break ground on anything: half again what it wants
/// to hold per head behind the fire, and a season that puts something back.
/// Building through a winter spends the only buffer there is.
pub fn can_afford_project(fuel: u32, population: usize, diverting: bool, growing: bool) -> bool {
    growing && diverting && stock_share(fuel, FUEL_PER_CITIZEN, population) >= BUILD_RESERVE_SHARE
}

/// Whether a log carried to `drop_off` joins the project rather than the fire.
/// Timber past what the project needs would be thrown away, so it burns.
pub fn log_goes_to_site(
    drop_off: IVec2,
    site_pos: IVec2,
    delivered: u32,
    building: Building,
) -> bool {
    drop_off == site_pos && delivered < building.rules().cost
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

/// Whether the colony is in any shape to take on another mouth: every need
/// comfortably met across most of it, and both stores at what it wants to hold
/// per head. Judging the stores per head rather than against a flat number is
/// what stops a colony growing straight through its own supply.
pub fn colony_thrives(shares: [f32; NEED_COUNT], fuel: u32, food: u32, population: usize) -> bool {
    shares.iter().all(|share| *share >= GROWTH_SHARE)
        && stock_share(fuel, FUEL_PER_CITIZEN, population) >= 1.0
        && stock_share(food, FOOD_PER_CITIZEN, population) >= 1.0
}

/// What the colony spends every tick at a given size, straight out of the
/// constants that do the spending: the fire's draw, and what every mouth eats.
/// Fuel and food are added as plain units of fetched stuff, which is what the
/// haulers' capacity is measured in too, so the comparison holds even though
/// the two are not interchangeable.
pub fn demand_per_tick(population: usize, boilers: usize) -> f32 {
    let fire = burn_amount(population, boilers) as f32 / BURN_EVERY as f32;
    let rules = NeedKind::Food.rules();
    let between_meals = (rules.high - rules.low) / rules.decay;
    let mouths = population as f32 * FOOD_PER_MEAL as f32 / between_meals;
    fire + mouths
}

/// Whether the colony could still feed and warm itself with one more mouth in
/// it, with a winter's margin left over. Stores say what has been fetched; this
/// asks whether anyone is left to keep fetching it, before the answer is no.
pub fn can_afford_a_mouth(hands: usize, per_hand: f32, population: usize, boilers: usize) -> bool {
    let supply = hands as f32 * per_hand;
    supply >= demand_per_tick(population + 1, boilers) * (1.0 + WINTER_MARGIN)
}

/// What the colony has actually been fetching, counted rather than assumed: one
/// reading a day of what came in and how many hands brought it.
#[derive(Resource, Default)]
pub struct Flow {
    today: u32,
    hauled: [u32; SEASON_DAYS],
    hands: [u32; SEASON_DAYS],
    next: usize,
    days: usize,
}

impl Flow {
    pub fn delivered(&mut self, units: u32) {
        self.today = self.today.saturating_add(units);
    }

    pub fn close_the_day(&mut self, hands: usize) {
        self.hauled[self.next] = self.today;
        self.hands[self.next] = hands as u32;
        self.today = 0;
        self.next = (self.next + 1) % SEASON_DAYS;
        self.days = self.days.saturating_add(1);
    }

    /// Units one hauler brings in per tick, over the days on record. `None`
    /// until a day has closed and somebody has actually hauled.
    pub fn per_hand(&self) -> Option<f32> {
        let days = self.days.min(SEASON_DAYS);
        if days == 0 {
            return None;
        }
        let hauled: u32 = self.hauled[..days].iter().sum();
        let hands: u32 = self.hands[..days].iter().sum();
        if hands == 0 {
            return None;
        }
        Some(hauled as f32 / (hands as f32 * ticks_per_day() as f32))
    }
}

/// What the colony held per head on each of the last season's days. Stocks say
/// how much has been fetched; only a trend says whether anyone can keep
/// fetching it, and a colony can starve with every store at its cap.
#[derive(Resource)]
pub struct Trend {
    history: [[f32; CARGO_COUNT]; SEASON_DAYS],
    next: usize,
    days: usize,
}

impl Default for Trend {
    fn default() -> Self {
        Trend {
            history: [[0.0; CARGO_COUNT]; SEASON_DAYS],
            next: 0,
            days: 0,
        }
    }
}

impl Trend {
    pub fn record(&mut self, shares: [f32; CARGO_COUNT]) {
        self.history[self.next] = shares;
        self.next = (self.next + 1) % SEASON_DAYS;
        self.days = self.days.saturating_add(1);
    }

    /// What was held a season back, once there has been a season to look back
    /// on. A colony in its first season has nothing to compare against.
    pub fn a_season_ago(&self) -> Option<[f32; CARGO_COUNT]> {
        (self.days >= SEASON_DAYS).then(|| self.history[self.next])
    }
}

/// Whether a store is holding up: no worse per head than it was a season ago,
/// or still at what the colony wants per head, which cannot rise further and so
/// must not be read as falling.
pub fn store_is_holding(now: f32, a_season_ago: f32) -> bool {
    now >= a_season_ago || now >= 1.0
}

pub fn stores_are_holding(now: [f32; CARGO_COUNT], then: Option<[f32; CARGO_COUNT]>) -> bool {
    match then {
        None => true,
        Some(then) => now
            .iter()
            .zip(then)
            .all(|(now, then)| store_is_holding(*now, then)),
    }
}

/// Whether the colony has hands to spare for another mouth. Stores say what it
/// has fetched, not whether anyone is left to fetch more, and a child is a
/// mouth for years before it is a pair of hands.
pub fn has_hands_to_spare(children: usize, population: usize) -> bool {
    population > 0 && (children as f32 / population as f32) <= MAX_DEPENDENT_SHARE
}

pub fn setup(mut commands: Commands, mut lineage: ResMut<Lineage>) {
    let houses = CITIZENS.div_ceil(HOUSE_CAPACITY);
    let sites: Vec<IVec2> = (0..houses).filter_map(plot_site).collect();
    for site in &sites {
        commands.spawn((Pos(*site), Structure(Building::House)));
    }

    let mut homes: Vec<IVec2> = Vec::new();
    for i in 0..CITIZENS {
        let Some(home) = free_home(&sites, &homes) else {
            break;
        };
        homes.push(home);
        let angle = i as f32 / CITIZENS as f32 * std::f32::consts::TAU;
        // Every seed in the colony comes from here, founders included, or a
        // newcomer would eventually be handed a founder's lifespan and rolls.
        let seed = lineage.next();
        commands.spawn((
            Pos(ring_pos(START_RING, angle)),
            Citizen {
                needs: Needs::founder(i, CITIZENS),
                upbringing: Upbringing::grown(seed),
                age: founder_age(i, CITIZENS),
                lifespan: lifespan_of(seed),
                seed,
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

/// A year of life per year on the calendar, so the date on screen never
/// contradicts anyone's age. Children run faster or slower than that with how
/// well the colony is feeding and warming them.
pub fn aging(mut citizens: Query<&mut Citizen>) {
    let people: Vec<Needs> = citizens.iter().map(|citizen| citizen.needs).collect();
    let shares = met_shares(&people);
    let growing_up = maturation_rate(
        shares[NeedKind::Warmth as usize],
        shares[NeedKind::Food as usize],
    );
    let warmth = shares[NeedKind::Warmth as usize];
    let food = shares[NeedKind::Food as usize];
    for mut citizen in &mut citizens {
        let rate = if is_adult(citizen.age) {
            1.0
        } else {
            growing_up
        };
        citizen.age += per_year(rate);
        if !is_adult(citizen.age) {
            citizen.upbringing.observe(warmth, food);
        }
        let age = citizen.age;
        citizen.upbringing.settle_due(age);
        citizen.upbringing.catch_up(age, warmth, food);
    }
}

/// The middle of a colony, which is what one citizen's stat is read against.
/// Sorts in place because the caller owns the scratch and nobody else wants it.
pub fn median(values: &mut [f32]) -> f32 {
    if values.is_empty() {
        return FORMATION_NEUTRAL;
    }
    values.sort_by(f32::total_cmp);
    values[values.len() / 2]
}

pub fn advance_calendar(tick: Res<Tick>, mut calendar: ResMut<Calendar>) {
    *calendar = calendar_at(tick.0);
}

pub fn burn_fuel(
    tick: Res<Tick>,
    built: Res<Built>,
    mut generator: ResMut<Generator>,
    citizens: Query<&Citizen>,
) {
    if tick.0.is_multiple_of(BURN_EVERY) {
        let burned = burn_amount(
            citizens.iter().count(),
            built.of(Building::GeneratorUpgrade),
        );
        generator.fuel = generator.fuel.saturating_sub(burned);
    }
}

/// The air this tick: what the fire is putting out, and how cold the year has
/// made everything outside its reach.
pub fn advance_weather(
    tick: Res<Tick>,
    built: Res<Built>,
    generator: Res<Generator>,
    mut air: ResMut<Air>,
) {
    *air = Air {
        output: generator_output(generator.fuel, built.of(Building::GeneratorUpgrade)),
        ambient: ambient_at(tick.0),
    };
}

/// A day's growing back, in the seasons that allow it.
pub fn regrow_patches(tick: Res<Tick>, calendar: Res<Calendar>, mut patches: ResMut<Patches>) {
    if !tick.0.is_multiple_of(ticks_per_day()) {
        return;
    }
    let growing = is_growing_season(calendar.season);
    for patch in &mut patches.0 {
        patch.amount = regrowth_step(patch.amount, patch.cap, patch.kind, growing);
    }
}

/// One reading a day of what the colony holds per head, kept a season deep.
pub fn record_trend(
    tick: Res<Tick>,
    stores: Stores,
    citizens: Query<&Citizen>,
    mut trend: ResMut<Trend>,
    mut flow: ResMut<Flow>,
) {
    if !tick.0.is_multiple_of(ticks_per_day()) {
        return;
    }
    let population = citizens.iter().count();
    trend.record([
        stock_share(stores.generator.fuel, FUEL_PER_CITIZEN, population),
        stock_share(stores.granary.food, FOOD_PER_CITIZEN, population),
    ]);
    flow.close_the_day(citizens.iter().filter(|c| is_adult(c.age)).count());
}

pub fn count_buildings(mut built: ResMut<Built>, structures: Query<&Structure>) {
    let mut counts = [0usize; BUILDING_COUNT];
    for structure in &structures {
        counts[structure.0 as usize] += 1;
    }
    built.0 = counts;
}

/// Finishes the project in progress, or opens the next one on a free plot once
/// the colony has timber to spare.
pub fn construction(
    mut commands: Commands,
    mut council: Council,
    calendar: Res<Calendar>,
    generator: Res<Generator>,
    structures: Query<(&Pos, &Structure)>,
    mut citizens: Query<&mut Citizen>,
) {
    council.construction.diverting =
        update_diverting(council.construction.diverting, generator.fuel);

    if let Some(site) = &council.construction.site {
        if site.delivered >= site.building.rules().cost {
            commands.spawn((Pos(site.pos), Structure(site.building)));
            council.construction.site = None;
        }
        return;
    }
    if !can_afford_project(
        generator.fuel,
        citizens.iter().count(),
        council.construction.diverting,
        is_growing_season(calendar.season),
    ) {
        return;
    }

    let taken: Vec<IVec2> = structures.iter().map(|(pos, _)| pos.0).collect();
    let beds: Vec<IVec2> = structures
        .iter()
        .filter(|(_, structure)| structure.0 == Building::House)
        .map(|(pos, _)| pos.0)
        .collect();
    let homes: Vec<IVec2> = citizens.iter().map(|citizen| citizen.home).collect();
    let people: Vec<Needs> = citizens.iter().map(|citizen| citizen.needs).collect();
    let tally = tally_votes(&people, &council.mayor);
    let building = next_project(tally, free_home(&beds, &homes).is_some());
    council.ballot.tally = tally;
    // The ballot is counted, so the season it was counted over is over too.
    for mut citizen in &mut citizens {
        citizen.needs.forget_before_ballot();
    }
    if let Some(pos) = next_plot(&taken) {
        council.construction.site = Some(Site {
            pos,
            building,
            delivered: 0,
        });
    }
}

/// Children, one roll per couple. A colony that is warm, rested and fed with
/// stores behind it has them; how often depends on how much room is left,
/// which tails off instead of stopping dead the way a free-bed gate does.
pub fn colony_growth(
    mut commands: Commands,
    tick: Res<Tick>,
    mut lineage: ResMut<Lineage>,
    stores: Stores,
    ledger: Ledger,
    houses: Query<(&Pos, &Structure)>,
    citizens: Query<&Citizen>,
) {
    if !tick.0.is_multiple_of(BIRTH_EVERY) {
        return;
    }
    let people: Vec<Needs> = citizens.iter().map(|citizen| citizen.needs).collect();
    let fuel = stores.generator.fuel;
    let food = stores.granary.food;
    if !colony_thrives(met_shares(&people), fuel, food, people.len()) {
        return;
    }
    // Full stores say nothing about whether anyone can keep them full.
    let holding = [
        stock_share(fuel, FUEL_PER_CITIZEN, people.len()),
        stock_share(food, FOOD_PER_CITIZEN, people.len()),
    ];
    if !stores_are_holding(holding, ledger.trend.a_season_ago()) {
        return;
    }
    // And whether there would be hands enough for one more, which no store can
    // say: a colony can sit on full stores it has nobody left to refill.
    if let Some(per_hand) = ledger.flow.per_hand()
        && !can_afford_a_mouth(
            citizens.iter().filter(|c| is_adult(c.age)).count(),
            per_hand,
            people.len(),
            ledger.built.of(Building::GeneratorUpgrade),
        )
    {
        return;
    }
    let sites: Vec<IVec2> = houses
        .iter()
        .filter(|(_, structure)| structure.0 == Building::House)
        .map(|(pos, _)| pos.0)
        .collect();
    let mut homes: Vec<IVec2> = citizens.iter().map(|citizen| citizen.home).collect();
    let ages: Vec<f32> = citizens.iter().map(|citizen| citizen.age).collect();
    let children = ages.iter().filter(|age| !is_adult(**age)).count();
    if !has_hands_to_spare(children, ages.len()) {
        return;
    }
    let pairs = couples(&ages);
    let chance = birth_chance_per_check(birth_chance_per_season(spare_beds(&sites, &homes), pairs));

    // Every couple rolls against the same snapshot, so a birth part way through
    // does not shift the rolls of the couples after it.
    let round = lineage.0;
    let expecting = (0..pairs)
        .filter(|pair| {
            noise(
                round.wrapping_add(*pair as u64),
                BIRTH_SALT.wrapping_add(tick.0),
            ) < chance
        })
        .count();

    for _ in 0..expecting {
        let Some(home) = free_home(&sites, &homes) else {
            break;
        };
        homes.push(home);
        let seed = lineage.next();
        commands.spawn((
            Pos(CENTER),
            Citizen {
                needs: Needs::newcomer(),
                upbringing: Upbringing::born(seed),
                age: 0.0,
                lifespan: lifespan_of(seed),
                seed,
                home,
                carrying: None,
                hauling: Cargo::Wood,
            },
        ));
    }
}

pub fn citizen_ai(
    mut commands: Commands,
    tick: Res<Tick>,
    air: Res<Air>,
    built: Res<Built>,
    mut flow: ResMut<Flow>,
    mut colony: Colony,
    mut citizens: Query<(Entity, &mut Pos, &mut Citizen)>,
) {
    let air = *air;
    let site_pos = colony.construction.site.as_ref().map(|site| site.pos);
    let population = citizens.iter().count();
    // Who is already committed to fetching what, kept up to date as citizens
    // change their minds, so each decision this tick sees the ones before it.
    let mut inbound = [0usize; CARGO_COUNT];
    for (_, _, citizen) in &citizens {
        if is_adult(citizen.age) {
            inbound[citizen.hauling as usize] += 1;
        }
    }
    // Once a day the cold gets its roll against whoever it has found.
    let cold_night = tick.0.is_multiple_of(ticks_per_day());

    for (entity, mut pos, mut citizen) in &mut citizens {
        let at_home = pos.0 == citizen.home;
        let at_centre = (pos.0 - CENTER).abs().max_element() <= 1;
        let grown = is_adult(citizen.age);
        let duty = choose_duty(&citizen.needs, citizen.carrying, grown);

        // Eating is what makes the food need met this tick, so it happens before
        // the needs are stepped.
        let eating = takes_a_meal(&citizen.needs, at_centre, colony.granary.food);
        if eating {
            colony.granary.food -= FOOD_PER_MEAL;
        }
        let warm = air.heat_at(pos.0) >= 0.0;
        let met = [warm, duty == Duty::Rest && at_home, eating];
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
        // The snap is the air they are standing in, not how desperate they are.
        // Rolling only once a citizen is already freezing would mean frailty
        // never got to decide anything.
        if cold_night && !warm {
            let roll = noise(citizen.seed, COLD_SNAP_SALT.wrapping_add(tick.0));
            if cold_takes(hardiness(citizen.age, citizen.lifespan), roll) {
                commands.entity(entity).despawn();
                continue;
            }
        }

        let supply = Supply {
            fuel: colony.generator.fuel,
            food: colony.granary.food,
            inbound,
            population,
            huts: built.of(Building::HuntersHut),
        };
        if let Some(cargo) = citizen.carrying {
            let drop_off = delivery_target(cargo, colony.construction.diverting, site_pos);
            if (pos.0 - drop_off).abs().max_element() <= 1 {
                match (cargo, colony.construction.site.as_mut()) {
                    (Cargo::Wood, Some(site))
                        if log_goes_to_site(drop_off, site.pos, site.delivered, site.building) =>
                    {
                        site.delivered += 1;
                    }
                    (Cargo::Wood, _) => colony.generator.fuel += 1,
                    (Cargo::Food, _) => {
                        colony.granary.food += food_yield(built.of(Building::HuntersHut))
                    }
                }
                flow.delivered(match cargo {
                    Cargo::Wood => 1,
                    Cargo::Food => food_yield(built.of(Building::HuntersHut)),
                });
                citizen.carrying = None;
                let next = haul_choice(citizen.hauling, supply);
                if next != citizen.hauling {
                    inbound[citizen.hauling as usize] -= 1;
                    inbound[next as usize] += 1;
                    citizen.hauling = next;
                }
            }
        }

        let source = gather_source(
            &colony.patches,
            citizen.hauling,
            pos.0,
            supply.wants(citizen.hauling.other()),
        );
        if duty == Duty::Gather
            && let Some((cell, kind)) = source
            && cell == pos.0
        {
            take_from_patch(&mut colony.patches, cell);
            citizen.carrying = Some(kind);
        }

        // Handing a load over or picking one up flips this tick's duty; nothing
        // else about the citizen has changed since it was chosen.
        let duty = choose_duty(&citizen.needs, citizen.carrying, grown);
        // Only a working citizen has working hours for the cold to take.
        if grown {
            citizen.needs.spend(duty == Duty::WarmUp);
        }
        let drop_off = citizen.carrying.map_or(CENTER, |cargo| {
            delivery_target(cargo, colony.construction.diverting, site_pos)
        });
        let target = duty_target(
            duty,
            air,
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
    fn every_plot() -> Vec<IVec2> {
        (0usize..)
            .map(plot_site)
            .take_while(Option::is_some)
            .flatten()
            .collect()
    }

    /// The air with the fire at a given stock, on a day of average cold.
    fn air(fuel: u32, upgrades: usize) -> Air {
        Air {
            output: generator_output(fuel, upgrades),
            ambient: AMBIENT_MEAN,
        }
    }

    fn need_at(level: f32) -> Need {
        Need {
            level,
            pressing: false,
            burden: 0.0,
        }
    }

    fn set(needs: &mut Needs, kind: NeedKind, level: f32, pressing: bool) {
        needs.needs[kind as usize] = Need {
            level,
            pressing,
            burden: shortfall_of(kind, level).max(0.0),
        };
    }

    #[test]
    fn heat_falls_off_with_distance() {
        let lit = air(FULL_BURN_FUEL, 0);
        let near = lit.heat_at(CENTER + IVec2::new(1, 0));
        let far = lit.heat_at(CENTER + IVec2::new(10, 0));
        assert!(near > far);
        assert!(lit.heat_at(CENTER) > 0.0);
    }

    #[test]
    fn heat_is_ambient_when_generator_is_off() {
        let dead = air(0, 0);
        assert_eq!(dead.heat_at(CENTER), AMBIENT_MEAN);
        assert_eq!(dead.heat_at(CENTER + IVec2::new(5, 5)), AMBIENT_MEAN);
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
        assert_eq!(generator_output(0, 0), 0.0);
        assert!(generator_output(FULL_BURN_FUEL / 2, 0) < generator_output(FULL_BURN_FUEL, 0));
        assert_eq!(generator_output(FULL_BURN_FUEL, 0), GENERATOR_HEAT);
        assert_eq!(
            generator_output(FULL_BURN_FUEL * 10, 0),
            GENERATOR_HEAT,
            "a full stock cannot be burned faster than the grate allows"
        );
    }

    #[test]
    fn the_warm_zone_shrinks_as_the_stock_runs_down() {
        let warm_radius = |fuel| {
            let lit = air(fuel, 0);
            (0..=R)
                .filter(|d| lit.heat_at(CENTER + IVec2::new(*d, 0)) > 0.0)
                .count()
        };
        assert!(warm_radius(FULL_BURN_FUEL) > warm_radius(FULL_BURN_FUEL / 2));
        assert!(warm_radius(FULL_BURN_FUEL / 2) > warm_radius(0));
        assert_eq!(warm_radius(0), 0, "a dead generator warms nothing");
    }

    #[test]
    fn citizens_fall_back_to_their_own_roof_when_the_square_goes_cold() {
        let home = IVec2::new(3, 4);
        assert_eq!(air(FULL_BURN_FUEL, 0).warmth_target(home), CENTER);
        assert_eq!(air(0, 0).warmth_target(home), home);
    }

    #[test]
    fn shortfall_reads_zero_at_the_high_mark_and_one_at_the_low_one() {
        for kind in NEEDS {
            let rules = kind.rules();
            let mut needs = Needs::newcomer();
            set(&mut needs, kind, rules.high, false);
            assert_eq!(needs.shortfall(kind), 0.0, "{kind:?} at the high mark");
            set(&mut needs, kind, rules.low, true);
            assert_eq!(needs.shortfall(kind), 1.0, "{kind:?} at the low mark");
            set(&mut needs, kind, 0.0, true);
            assert!(
                needs.shortfall(kind) > 1.0,
                "{kind:?} can be worse than the band"
            );
        }
    }

    #[test]
    fn needs_living_in_different_bands_still_compare() {
        let mut needs = Needs::newcomer();
        for kind in NEEDS {
            set(&mut needs, kind, kind.rules().low, true);
        }
        let first = needs.shortfall(NEEDS[0]);
        for kind in NEEDS {
            assert_eq!(
                needs.shortfall(kind),
                first,
                "every need at its own low mark is equally short"
            );
        }
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
                burden: 0.0,
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
                        pressing: true,
                        burden: 0.0
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
    fn equally_short_needs_break_toward_the_first_listed_one() {
        let mut needs = Needs::newcomer();
        for kind in [NeedKind::Rest, NeedKind::Food] {
            set(&mut needs, kind, kind.rules().low, true);
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
            choose_duty(&needs, Some(Cargo::Wood), true),
            Duty::Deliver,
            "wood is dropped off before bed"
        );
        assert_eq!(choose_duty(&needs, None, true), Duty::Rest);

        set(&mut needs, NeedKind::Food, 1.0, true);
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Wood), true),
            Duty::Eat,
            "a starving citizen puts the load down"
        );
    }

    #[test]
    fn a_citizen_with_nothing_pressing_goes_to_work() {
        let needs = Needs::newcomer();
        assert_eq!(choose_duty(&needs, None, true), Duty::Gather);
        assert_eq!(choose_duty(&needs, Some(Cargo::Food), true), Duty::Deliver);
    }

    #[test]
    fn each_duty_walks_to_its_own_destination() {
        let home = IVec2::new(3, 4);
        let drop_off = IVec2::new(7, 8);
        let patch = IVec2::new(1, 1);
        let lit = air(FULL_BURN_FUEL, 0);
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
            duty_target(Duty::Gather, air(FULL_BURN_FUEL, 0), home, drop_off, None),
            CENTER,
            "while the fire burns, idle citizens huddle around it"
        );
        assert_eq!(
            duty_target(Duty::Gather, air(0, 0), home, drop_off, None),
            home,
            "once it is out, the only shelter left is their own roof"
        );
    }

    /// Stock levels that put the two stores exactly `gap` shares apart, with
    /// wood ahead when `gap` is positive. Pin one store on its target rather
    /// than piling it high: a store far above target swamps the band, and a
    /// test written that way passes whatever the band does.
    fn stores_apart(population: usize, gap: f32) -> (u32, u32) {
        let fuel = population as f32 * FUEL_PER_CITIZEN;
        let food = population as f32 * FOOD_PER_CITIZEN * (1.0 - gap);
        (fuel.round() as u32, food.round() as u32)
    }

    /// The colony as a hauler sees it: stores, workforce, huts, nobody yet on
    /// their way home.
    fn supply(fuel: u32, food: u32, population: usize, huts: usize) -> Supply {
        Supply {
            fuel,
            food,
            inbound: [0; CARGO_COUNT],
            population,
            huts,
        }
    }

    #[test]
    fn haulers_turn_to_whichever_store_has_fallen_clearly_behind() {
        assert_eq!(haul_choice(Cargo::Wood, supply(200, 1, 30, 0)), Cargo::Food);
        assert_eq!(haul_choice(Cargo::Food, supply(1, 200, 30, 0)), Cargo::Wood);
    }

    #[test]
    fn what_is_already_on_its_way_counts_towards_the_store() {
        // Fuel sitting exactly on its target, game all but gone.
        let short = supply(39, 1, 30, 0);
        assert_eq!(
            haul_choice(Cargo::Wood, short),
            Cargo::Food,
            "with nobody fetching game the shortage pulls a hauler over"
        );
        let mut answered = short;
        answered.inbound[Cargo::Food as usize] = 20;
        assert_eq!(
            haul_choice(Cargo::Wood, answered),
            Cargo::Wood,
            "with the shortage already answered it pulls nobody else"
        );
    }

    #[test]
    fn a_haul_in_flight_is_worth_what_it_will_bring_back() {
        let mut with_huts = supply(39, 1, 30, 3);
        let mut without = supply(39, 1, 30, 0);
        with_huts.inbound[Cargo::Food as usize] = 6;
        without.inbound[Cargo::Food as usize] = 6;
        assert_eq!(
            haul_choice(Cargo::Wood, without),
            Cargo::Food,
            "six plain hauls do not cover the gap"
        );
        assert_eq!(
            haul_choice(Cargo::Wood, with_huts),
            Cargo::Wood,
            "the same six do once huts make each one worth four"
        );
    }

    #[test]
    fn a_projection_is_the_store_plus_what_is_walking_towards_it() {
        assert_eq!(projected_stock(10, 0, 3), 10);
        assert_eq!(projected_stock(10, 4, 3), 22);
        assert_eq!(projected_stock(0, 0, 1), 0);
    }

    #[test]
    fn a_hauler_holds_its_kind_until_the_other_store_clears_the_band() {
        let pop = 30;
        let band = haul_switch_margin(pop, 0);
        let (fuel, food) = stores_apart(pop, band / 2.0);
        assert_eq!(
            haul_choice(Cargo::Wood, supply(fuel, food, pop, 0)),
            Cargo::Wood,
            "inside the band the workforce does not turn around"
        );
        let (fuel, food) = stores_apart(pop, band * 2.0);
        assert_eq!(
            haul_choice(Cargo::Wood, supply(fuel, food, pop, 0)),
            Cargo::Food,
            "past the band it does"
        );
    }

    #[test]
    fn the_band_works_the_same_way_round() {
        let pop = 30;
        let band = haul_switch_margin(pop, 0);
        let (fuel, food) = stores_apart(pop, -band / 2.0);
        assert_eq!(
            haul_choice(Cargo::Food, supply(fuel, food, pop, 0)),
            Cargo::Food
        );
        let (fuel, food) = stores_apart(pop, -band * 2.0);
        assert_eq!(
            haul_choice(Cargo::Food, supply(fuel, food, pop, 0)),
            Cargo::Wood
        );
    }

    #[test]
    fn evenly_stocked_stores_leave_every_hauler_where_they_are() {
        let pop = 30;
        let (fuel, food) = stores_apart(pop, 0.0);
        assert_eq!(
            haul_choice(Cargo::Wood, supply(fuel, food, pop, 0)),
            Cargo::Wood
        );
        assert_eq!(
            haul_choice(Cargo::Food, supply(fuel, food, pop, 0)),
            Cargo::Food
        );
    }

    #[test]
    fn haul_choice_survives_an_empty_colony() {
        assert_eq!(haul_choice(Cargo::Wood, supply(0, 0, 0, 0)), Cargo::Wood);
        assert_eq!(haul_choice(Cargo::Food, supply(0, 0, 0, 0)), Cargo::Food);
    }

    #[test]
    fn the_band_widens_with_every_hut_until_it_hits_its_ceiling() {
        // A big workforce keeps one haul's swing small, so the ceiling stays
        // out of the way and the widening is visible on its own.
        let roomy = 200;
        assert!(haul_switch_margin(roomy, 1) > haul_switch_margin(roomy, 0));
        assert!(haul_switch_margin(roomy, 3) > haul_switch_margin(roomy, 1));
        assert_eq!(
            haul_switch_margin(30, 3),
            HAUL_SWITCH_MAX,
            "and stops once it would swallow the decision"
        );
    }

    #[test]
    fn the_band_scales_with_the_workforce_and_never_degenerates() {
        assert!(
            haul_switch_margin(60, 0) < haul_switch_margin(30, 0),
            "with twice the haulers one trip moves the store half as far"
        );
        let band = haul_switch_margin(30, 0);
        assert!(band > 0.0 && band.is_finite());
        assert!(
            haul_switch_margin(0, 0).is_finite(),
            "an empty colony is not a divide by zero"
        );
    }

    #[test]
    fn a_hut_makes_the_workforce_slower_to_swing() {
        let pop = 30;
        let gap = haul_switch_margin(pop, 0) * 1.5;
        let (fuel, food) = stores_apart(pop, gap);
        assert_eq!(
            haul_choice(Cargo::Wood, supply(fuel, food, pop, 0)),
            Cargo::Food,
            "with no hut that gap is worth turning around for"
        );
        assert_eq!(
            haul_choice(Cargo::Wood, supply(fuel, food, pop, 2)),
            Cargo::Wood,
            "with huts standing the same gap is one haul away and not worth it"
        );
    }

    #[test]
    fn a_hauler_switches_kind_when_its_own_patches_are_stripped() {
        let mut patches = Patches(vec![
            Patch {
                pos: CENTER + IVec2::new(4, 0),
                kind: Cargo::Wood,
                amount: 0,
                cap: 0,
            },
            Patch {
                pos: CENTER + IVec2::new(0, 4),
                kind: Cargo::Food,
                amount: 5,
                cap: 5,
            },
        ]);
        let from = CENTER;
        assert_eq!(
            gather_source(&patches, Cargo::Wood, from, true),
            Some((CENTER + IVec2::new(0, 4), Cargo::Food)),
            "a stripped forest sends the hauler after game instead"
        );
        patches.0[1].amount = 0;
        assert_eq!(gather_source(&patches, Cargo::Wood, from, true), None);
    }

    #[test]
    fn a_hauler_prefers_its_own_kind_when_both_are_standing() {
        let patches = Patches(vec![
            Patch {
                pos: CENTER + IVec2::new(9, 0),
                kind: Cargo::Wood,
                amount: 5,
                cap: 5,
            },
            Patch {
                pos: CENTER + IVec2::new(0, 2),
                kind: Cargo::Food,
                amount: 5,
                cap: 5,
            },
        ]);
        assert_eq!(
            gather_source(&patches, Cargo::Wood, CENTER, true),
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
            cap: 2,
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
        assert!(!colony_thrives(shares, u32::MAX, u32::MAX, 0));
    }

    #[test]
    fn growth_needs_every_need_met_and_both_stores_at_target() {
        let full = [1.0; NEED_COUNT];
        let pop = 30;
        let fuel = (pop as f32 * FUEL_PER_CITIZEN).ceil() as u32;
        let food = (pop as f32 * FOOD_PER_CITIZEN).ceil() as u32;
        assert!(colony_thrives(full, fuel, food, pop));
        assert!(!colony_thrives(full, 0, food, pop), "no wood");
        assert!(!colony_thrives(full, fuel, 0, pop), "no food");
        for kind in NEEDS {
            let mut shares = full;
            shares[kind as usize] = GROWTH_SHARE - 0.1;
            assert!(
                !colony_thrives(shares, fuel, food, pop),
                "a colony short on {kind:?} must not grow"
            );
        }
    }

    #[test]
    fn what_counts_as_enough_grows_with_the_colony() {
        let full = [1.0; NEED_COUNT];
        let small = 20;
        let fuel = (small as f32 * FUEL_PER_CITIZEN).ceil() as u32;
        let food = (small as f32 * FOOD_PER_CITIZEN).ceil() as u32;
        assert!(colony_thrives(full, fuel, food, small));
        assert!(
            !colony_thrives(full, fuel, food, small * 2),
            "the same stores do not stretch to twice the mouths"
        );
    }

    #[test]
    fn every_building_costs_timber_and_can_be_told_apart() {
        let mut names = Vec::new();
        for building in BUILDINGS {
            let rules = building.rules();
            assert!(rules.cost > 0, "{building:?} must cost something");
            assert!(!rules.name.is_empty());
            assert!(!names.contains(&rules.name), "{building:?} reuses a name");
            names.push(rules.name);
        }
        assert_eq!(names.len(), BUILDING_COUNT);
    }

    #[test]
    fn every_need_has_a_building_that_answers_it() {
        let mut answers = Vec::new();
        for kind in NEEDS {
            let building = building_for(kind);
            assert!(!answers.contains(&building), "{kind:?} shares a remedy");
            answers.push(building);
        }
        assert_eq!(
            answers.len(),
            BUILDING_COUNT,
            "every building is somebody's remedy"
        );
    }

    fn mayor_leaning(building: Building, weight: f32) -> Mayor {
        let mut bias = [0.0; BUILDING_COUNT];
        bias[building as usize] = weight;
        Mayor { bias }
    }

    /// A citizen whose only complaint is `kind`, sunk to its low mark.
    fn voter_short_on(kind: NeedKind) -> Needs {
        let mut needs = Needs::newcomer();
        for other in NEEDS {
            set(&mut needs, other, other.rules().high, false);
        }
        set(&mut needs, kind, kind.rules().low, true);
        needs
    }

    #[test]
    fn the_tally_counts_one_voice_each() {
        let people = [
            voter_short_on(NeedKind::Food),
            voter_short_on(NeedKind::Food),
            voter_short_on(NeedKind::Rest),
        ];
        let tally = tally_votes(&people, &Mayor::default());
        assert_eq!(tally[Building::HuntersHut as usize], 2.0);
        assert_eq!(tally[Building::House as usize], 1.0);
        assert_eq!(tally[Building::GeneratorUpgrade as usize], 0.0);
    }

    #[test]
    fn a_neutral_mayor_leaves_the_ballot_alone() {
        let people = [voter_short_on(NeedKind::Warmth)];
        let plain = tally_votes(&people, &Mayor::default());
        assert_eq!(plain[Building::GeneratorUpgrade as usize], 1.0);
        assert_eq!(winner(plain), Building::GeneratorUpgrade);
    }

    #[test]
    fn the_mayor_leans_on_the_tally_without_casting_a_vote() {
        let people = [voter_short_on(NeedKind::Food)];
        let tally = tally_votes(&people, &mayor_leaning(Building::House, 2.0));
        assert_eq!(
            tally[Building::HuntersHut as usize],
            1.0,
            "the vote still counts"
        );
        assert_eq!(tally[Building::House as usize], 2.0);
        assert_eq!(
            winner(tally),
            Building::House,
            "and the office can outweigh it"
        );
    }

    #[test]
    fn a_mayor_can_lean_against_a_building_as_well_as_for_it() {
        let people = [
            voter_short_on(NeedKind::Food),
            voter_short_on(NeedKind::Rest),
        ];
        let mayor = mayor_leaning(Building::HuntersHut, -2.0);
        assert_eq!(winner(tally_votes(&people, &mayor)), Building::House);
    }

    #[test]
    fn an_even_ballot_resolves_in_table_order() {
        assert_eq!(winner([0.0; BUILDING_COUNT]), BUILDINGS[0]);
        let mut tied = [0.0; BUILDING_COUNT];
        tied[Building::House as usize] = 3.0;
        tied[Building::HuntersHut as usize] = 3.0;
        assert_eq!(winner(tied), Building::House, "ties resolve in table order");
    }

    #[test]
    fn beds_come_first_when_there_are_none_to_be_had() {
        let mut ballot = [0.0; BUILDING_COUNT];
        ballot[Building::HuntersHut as usize] = 99.0;
        assert_eq!(
            next_project(ballot, false),
            Building::House,
            "nothing else the colony builds will let it grow"
        );
        assert_eq!(next_project(ballot, true), Building::HuntersHut);
    }

    #[test]
    fn a_project_waits_until_the_fire_has_a_reserve_behind_it() {
        let population = 30;
        let reserve = (population as f32 * FUEL_PER_CITIZEN * BUILD_RESERVE_SHARE).ceil() as u32;
        assert!(can_afford_project(reserve, population, true, true));
        assert!(
            !can_afford_project(reserve / 2, population, true, true),
            "the reserve is not spent on building"
        );
        assert!(
            !can_afford_project(reserve * 10, population, false, true),
            "a colony that is not sparing wood does not break ground"
        );
        assert!(
            !can_afford_project(reserve * 10, population, true, false),
            "and nothing is begun in a season that grows nothing back"
        );
        assert!(
            !can_afford_project(reserve, population * 3, true, true),
            "the same pile is not a reserve for three times the mouths"
        );
    }

    #[test]
    fn huts_stop_helping_past_a_handful() {
        assert!(food_yield(USEFUL_HUTS) > food_yield(USEFUL_HUTS - 1));
        assert_eq!(
            food_yield(USEFUL_HUTS * 5),
            food_yield(USEFUL_HUTS),
            "a twentieth shed does not dress a carcass any faster"
        );
    }

    #[test]
    fn a_boiler_burns_hotter_and_reaches_further() {
        let plain = air(FULL_BURN_FUEL, 0);
        let upgraded = air(FULL_BURN_FUEL, 1);
        assert!(upgraded.output > plain.output);
        let reach = |lit: Air| {
            (0..=R)
                .filter(|d| lit.heat_at(CENTER + IVec2::new(*d, 0)) > 0.0)
                .count()
        };
        assert!(
            reach(upgraded) > reach(plain),
            "a hotter fire warms more ground"
        );
        assert_eq!(
            generator_output(0, 3),
            0.0,
            "boilers cannot burn what is not there"
        );
    }

    #[test]
    fn a_boiler_is_paid_for_in_fuel_forever() {
        assert!(burn_amount(30, 1) > burn_amount(30, 0));
        assert!(burn_amount(30, 2) > burn_amount(30, 1));
    }

    #[test]
    fn each_hut_adds_to_what_a_haul_of_game_brings_back() {
        assert_eq!(food_yield(0), 1);
        assert!(food_yield(1) > food_yield(0));
        assert!(food_yield(3) > food_yield(1));
    }

    #[test]
    fn every_building_type_draws_on_the_same_run_of_plots() {
        let first = plot_site(0).expect("plot 0 exists");
        let second = plot_site(1).expect("plot 1 exists");
        assert_eq!(
            next_plot(&[first]),
            Some(second),
            "a hut occupies a plot a house could have had"
        );
    }

    #[test]
    fn logs_past_what_a_project_needs_go_on_the_fire() {
        let site = IVec2::new(5, 5);
        for building in BUILDINGS {
            let cost = building.rules().cost;
            assert!(log_goes_to_site(site, site, cost - 1, building));
            assert!(
                !log_goes_to_site(site, site, cost, building),
                "a finished {building:?} must not swallow timber"
            );
            assert!(
                !log_goes_to_site(CENTER, site, 0, building),
                "wood headed for the fire stays there"
            );
        }
    }

    #[test]
    fn a_bigger_city_burns_more_fuel_but_the_fire_never_stalls() {
        assert!(
            burn_amount(0, 0) >= 1,
            "the generator always burns something"
        );
        assert!(burn_amount(100, 0) > burn_amount(10, 0));
        assert!(
            burn_amount(10, 0) <= burn_amount(11, 0),
            "burn must not drop as the city grows"
        );
    }

    #[test]
    fn plots_are_distinct_and_fit_on_the_map() {
        let sites = every_plot();
        assert!(sites.len() >= 40, "the colony needs room to grow into");
        for (i, a) in sites.iter().enumerate() {
            assert!(a.x >= 0 && a.x <= R * 2 && a.y >= 0 && a.y <= R * 2);
            for b in &sites[i + 1..] {
                assert_ne!(a, b, "two houses landed on the same cell");
            }
        }
    }

    #[test]
    fn plots_never_cover_the_generator_or_a_harvest_patch() {
        let patches: Vec<IVec2> = patch_sites().into_iter().map(|patch| patch.pos).collect();
        for site in every_plot() {
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
    fn plots_run_out_beyond_the_buildable_rings() {
        let last = (0..)
            .take(10_000)
            .take_while(|i| plot_site(*i).is_some())
            .count();
        assert!(last > 0, "at least one site must be buildable");
        assert!(plot_site(last).is_none(), "sites must end, not wrap around");
    }

    #[test]
    fn free_home_fills_each_house_to_capacity_before_the_next() {
        let sites: Vec<IVec2> = (0..2).filter_map(plot_site).collect();
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
        let sites: Vec<IVec2> = (0..2).filter_map(plot_site).collect();
        let homes: Vec<IVec2> = sites
            .iter()
            .flat_map(|s| std::iter::repeat_n(*s, HOUSE_CAPACITY))
            .collect();
        assert_eq!(free_home(&sites, &homes), None);
    }

    #[test]
    fn the_next_building_plot_skips_the_houses_already_standing() {
        let first = plot_site(0).expect("site 0 exists");
        let second = plot_site(1).expect("site 1 exists");
        assert_eq!(next_plot(&[]), Some(first));
        assert_eq!(next_plot(&[first]), Some(second));
        assert_eq!(next_plot(&[second]), Some(first), "gaps get filled first");
    }

    #[test]
    fn the_next_building_plot_runs_out_once_the_rings_are_built_out() {
        let all = every_plot();
        assert!(!all.is_empty());
        assert_eq!(next_plot(&all), None);
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

    #[test]
    fn founders_do_not_all_get_hungry_in_the_same_hour() {
        let low = NeedKind::Food.rules().low;
        let levels: Vec<f32> = (0..CITIZENS)
            .map(|i| Needs::founder(i, CITIZENS).level(NeedKind::Food))
            .collect();
        for level in &levels {
            assert!(*level > low, "no founder starts the run already hungry");
            assert!(*level <= NEED_MAX);
        }
        let highest = levels.iter().copied().fold(f32::MIN, f32::max);
        let lowest = levels.iter().copied().fold(f32::MAX, f32::min);
        assert!(
            highest - lowest > (NEED_MAX - low) / 2.0,
            "the stagger must cover most of the band, or they still queue together"
        );
    }

    #[test]
    fn a_founder_is_a_newcomer_in_every_other_respect() {
        let founder = Needs::founder(0, CITIZENS);
        let newcomer = Needs::newcomer();
        assert_eq!(
            founder.level(NeedKind::Warmth),
            newcomer.level(NeedKind::Warmth)
        );
        assert_eq!(
            founder.level(NeedKind::Rest),
            newcomer.level(NeedKind::Rest)
        );
    }

    #[test]
    fn a_party_of_one_does_not_divide_by_zero() {
        assert!(Needs::founder(0, 0).level(NeedKind::Food).is_finite());
    }

    #[test]
    fn a_meal_needs_hunger_a_granary_and_something_in_it() {
        let mut hungry = Needs::newcomer();
        set(
            &mut hungry,
            NeedKind::Food,
            NeedKind::Food.rules().low,
            true,
        );
        assert!(takes_a_meal(&hungry, true, FOOD_PER_MEAL));
        assert!(
            !takes_a_meal(&hungry, true, 0),
            "an empty granary feeds nobody"
        );
        assert!(
            !takes_a_meal(&hungry, false, FOOD_PER_MEAL),
            "and it has to be walked to"
        );
        assert!(
            !takes_a_meal(&Needs::newcomer(), true, FOOD_PER_MEAL),
            "a fed citizen does not help themselves"
        );
    }

    #[test]
    fn a_starving_citizen_eats_even_while_something_worse_is_on_their_mind() {
        let mut desperate = Needs::newcomer();
        set(&mut desperate, NeedKind::Food, 25.0, true);
        set(&mut desperate, NeedKind::Warmth, 2.0, true);
        assert!(
            desperate.shortfall(NeedKind::Warmth) > desperate.shortfall(NeedKind::Food),
            "this citizen is worse off for cold than for hunger"
        );
        assert_eq!(
            choose_duty(&desperate, None, true),
            Duty::WarmUp,
            "so the cold is what they walk towards"
        );
        assert!(
            takes_a_meal(&desperate, true, FOOD_PER_MEAL),
            "but nobody starves standing on the granary"
        );
    }

    #[test]
    fn every_founder_starts_inside_the_warm_ring() {
        let lit = air(START_FUEL, 0);
        for i in 0..CITIZENS {
            let angle = i as f32 / CITIZENS as f32 * std::f32::consts::TAU;
            let start = ring_pos(START_RING, angle);
            assert!(
                lit.heat_at(start) >= 0.0,
                "a founder must not start out in the cold"
            );
        }
    }

    #[test]
    fn noise_is_uniform_enough_to_roll_against_and_never_repeats_itself() {
        let mut buckets = [0usize; 10];
        for seed in 0..2000u64 {
            let value = noise(seed, 7);
            assert!((0.0..1.0).contains(&value), "noise left the unit interval");
            buckets[(value * 10.0) as usize] += 1;
        }
        for (i, count) in buckets.iter().enumerate() {
            assert!(*count > 100, "bucket {i} holds only {count} of 2000");
        }
        assert_ne!(noise(1, 1), noise(1, 2), "a different roll differs");
        assert_ne!(noise(1, 1), noise(2, 1), "a different citizen differs");
    }

    #[test]
    fn noise_replays_the_same_way() {
        assert_eq!(noise(42, 3), noise(42, 3));
    }

    #[test]
    fn a_lifespan_is_near_the_expected_one_but_never_exactly_it() {
        let spans: Vec<f32> = (0..500u64).map(lifespan_of).collect();
        for span in &spans {
            let drift = (span - LIFESPAN_BASE).abs() / LIFESPAN_BASE;
            assert!(
                drift <= LIFESPAN_SPREAD + 1e-6,
                "{span} is outside the spread"
            );
        }
        let highest = spans.iter().copied().fold(f32::MIN, f32::max);
        let lowest = spans.iter().copied().fold(f32::MAX, f32::min);
        assert!(
            highest - lowest > LIFESPAN_BASE * LIFESPAN_SPREAD,
            "lifespans must actually vary, or the deaths come in a cohort"
        );
    }

    #[test]
    fn hardiness_holds_until_frailty_then_falls_away() {
        let span = LIFESPAN_BASE;
        assert_eq!(hardiness(0.0, span), 1.0);
        assert_eq!(
            hardiness(FRAILTY_ONSET, span),
            1.0,
            "nobody frails on a birthday"
        );
        assert!(hardiness(FRAILTY_ONSET + 5.0, span) < 1.0);
        assert!(hardiness(FRAILTY_ONSET + 10.0, span) < hardiness(FRAILTY_ONSET + 5.0, span));
        assert_eq!(hardiness(span, span), 0.0);
        assert_eq!(
            hardiness(span * 2.0, span),
            0.0,
            "hardiness does not go negative"
        );
    }

    #[test]
    fn a_shorter_lifespan_frails_faster() {
        let age = FRAILTY_ONSET + 5.0;
        assert!(
            hardiness(age, LIFESPAN_BASE * 0.9) < hardiness(age, LIFESPAN_BASE * 1.1),
            "the same age is harder on someone with less span left"
        );
    }

    #[test]
    fn the_cold_spares_the_hale_and_finds_the_frail() {
        assert!(!cold_takes(1.0, 0.0), "full hardiness shrugs off any night");
        assert!(
            cold_takes(0.0, 0.0),
            "no hardiness and the worst roll is fatal"
        );
        let unlucky = 0.0;
        assert!(
            !cold_takes(0.9, COLD_SNAP_LETHALITY),
            "a roll at the lethality bound is survived"
        );
        assert!(cold_takes(0.5, unlucky));
    }

    #[test]
    fn a_cold_night_is_survivable_far_more_often_than_not() {
        let frail = 0.5;
        let taken = (0..1000u64)
            .filter(|seed| cold_takes(frail, noise(*seed, 11)))
            .count();
        assert!(taken > 0, "a frail citizen must be at some risk");
        assert!(
            taken < 200,
            "one cold night in five would be a massacre, not a winter: {taken} of 1000"
        );
    }

    #[test]
    fn childhood_ends_at_adulthood_and_not_before() {
        assert!(!is_adult(0.0));
        assert!(!is_adult(ADULT_AGE - 0.1));
        assert!(is_adult(ADULT_AGE));
        assert!(is_adult(LIFESPAN_BASE));
    }

    #[test]
    fn a_colony_with_something_to_spare_raises_children_faster() {
        assert!(maturation_rate(1.0, 1.0) > maturation_rate(0.0, 0.0));
        assert!(
            maturation_rate(0.0, 0.0) > 0.0,
            "children still grow in a hard year"
        );
        assert!(maturation_rate(1.0, 0.0) > maturation_rate(0.0, 0.0));
        assert!(maturation_rate(0.0, 1.0) > maturation_rate(0.0, 0.0));
    }

    #[test]
    fn couples_are_counted_from_grown_citizens_of_an_age_for_it() {
        let grown = [ADULT_AGE, ADULT_AGE + 1.0, ADULT_AGE + 2.0, ADULT_AGE + 3.0];
        assert_eq!(couples(&grown), 2);
        assert_eq!(couples(&[ADULT_AGE]), 0, "one citizen is not a couple");
        assert_eq!(
            couples(&[ADULT_AGE - 1.0; 8]),
            0,
            "children are not couples"
        );
        assert_eq!(couples(&[FERTILE_UNTIL + 1.0; 8]), 0, "nor are the elderly");
    }

    #[test]
    fn spare_beds_scale_the_birth_rate_instead_of_gating_it() {
        assert_eq!(birth_chance_per_season(0, 10), 0.0, "a full colony stops");
        let some = birth_chance_per_season(5, 10);
        let plenty = birth_chance_per_season(10, 10);
        assert!(some > 0.0 && some < plenty, "room scales the rate smoothly");
        assert_eq!(
            birth_chance_per_season(100, 10),
            plenty,
            "beyond one bed a couple, more room adds nothing"
        );
        assert_eq!(birth_chance_per_season(10, 0), 0.0, "nobody to have them");
    }

    #[test]
    fn a_seasonal_chance_spread_over_its_checks_keeps_its_rate() {
        let seasonal = 0.9;
        let per_check = birth_chance_per_check(seasonal);
        let checks = (ticks_per_season() / BIRTH_EVERY) as f32;
        assert!((per_check * checks - seasonal).abs() < 1e-4);
        assert!(per_check < seasonal, "one check is not the whole season");
    }

    #[test]
    fn founders_are_grown_and_spread_across_the_years() {
        let ages: Vec<f32> = (0..CITIZENS).map(|i| founder_age(i, CITIZENS)).collect();
        for age in &ages {
            assert!(is_adult(*age), "a founding party is not made of children");
            assert!(
                *age < LIFESPAN_BASE,
                "nor of people already past their span"
            );
        }
        let highest = ages.iter().copied().fold(f32::MIN, f32::max);
        let lowest = ages.iter().copied().fold(f32::MAX, f32::min);
        assert!(
            highest > FRAILTY_ONSET,
            "some founders must already be frail, or nobody ages inside a run"
        );
        assert!(
            highest - lowest > FRAILTY_ONSET / 2.0,
            "a founding party all of an age dies all at once"
        );
    }

    #[test]
    fn spare_beds_are_what_is_left_after_everyone_is_housed() {
        let sites: Vec<IVec2> = (0..2).filter_map(plot_site).collect();
        let beds = sites.len() * HOUSE_CAPACITY;
        assert_eq!(spare_beds(&sites, &[]), beds);
        let homes: Vec<IVec2> = std::iter::repeat_n(sites[0], HOUSE_CAPACITY).collect();
        assert_eq!(spare_beds(&sites, &homes), beds - HOUSE_CAPACITY);
        let full: Vec<IVec2> = sites
            .iter()
            .flat_map(|site| std::iter::repeat_n(*site, HOUSE_CAPACITY))
            .collect();
        assert_eq!(spare_beds(&sites, &full), 0);
        let crowded: Vec<IVec2> = std::iter::repeat_n(sites[0], beds + 4).collect();
        assert_eq!(
            spare_beds(&sites, &crowded),
            0,
            "an overfull colony has no room"
        );
    }

    #[test]
    fn a_child_stays_out_of_the_way_instead_of_going_to_work() {
        let calm = Needs::newcomer();
        assert_eq!(choose_duty(&calm, None, true), Duty::Gather);
        assert_eq!(
            choose_duty(&calm, None, false),
            Duty::Rest,
            "children are mouths, not hands, until they are grown"
        );
    }

    #[test]
    fn a_child_still_answers_its_own_needs() {
        let mut hungry = Needs::newcomer();
        set(
            &mut hungry,
            NeedKind::Food,
            NeedKind::Food.rules().low,
            true,
        );
        assert_eq!(choose_duty(&hungry, None, false), Duty::Eat);
        let mut cold = Needs::newcomer();
        set(
            &mut cold,
            NeedKind::Warmth,
            NeedKind::Warmth.rules().low,
            true,
        );
        assert_eq!(choose_duty(&cold, None, false), Duty::WarmUp);
    }

    #[test]
    fn no_two_citizens_are_ever_handed_the_same_seed() {
        let mut lineage = Lineage::default();
        let seeds: Vec<u64> = (0..CITIZENS * 4).map(|_| lineage.next()).collect();
        for (i, seed) in seeds.iter().enumerate() {
            for other in &seeds[i + 1..] {
                assert_ne!(
                    seed, other,
                    "shared seeds mean shared lifespans and shared rolls"
                );
            }
        }
    }

    #[test]
    fn seeds_are_handed_out_from_the_first_one() {
        let mut lineage = Lineage::default();
        assert_eq!(lineage.next(), 1, "there is no citizen zero");
    }

    /// The tick that lands on a given day of a given year.
    fn day_of(year: u64, day: u64) -> u64 {
        (year - 1) * ticks_per_year() + day * ticks_per_day()
    }

    #[test]
    fn the_year_is_mildest_at_midsummer_and_harshest_at_midwinter() {
        let year = SEVERITY_FULL_YEAR;
        let midsummer = ambient_at(day_of(year, DAYS_PER_SEASON + DAYS_PER_SEASON / 2));
        let midwinter = ambient_at(day_of(year, DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2));
        assert!(midsummer > midwinter, "summer must be the mild end");
        assert!(midwinter < AMBIENT_MEAN, "and winter the harsh one");
        for day in 0..days_per_year() {
            let air = ambient_at(day_of(year, day));
            assert!(air <= midsummer + 1e-3, "day {day} beat midsummer");
            assert!(air >= midwinter - 1e-3, "day {day} beat midwinter");
        }
    }

    #[test]
    fn the_air_drifts_rather_than_jumps() {
        let mut previous = ambient_at(0);
        for day in 1..days_per_year() * 3 {
            let air = ambient_at(day * ticks_per_day());
            assert!(
                (air - previous).abs() < AMBIENT_SWING / 4.0,
                "the air moved too far in a day, at day {day}"
            );
            previous = air;
        }
    }

    #[test]
    fn winter_pulls_the_freezing_line_inside_the_treeline() {
        let year = SEVERITY_FULL_YEAR;
        let midwinter = Air {
            output: generator_output(FULL_BURN_FUEL, 0),
            ambient: ambient_at(day_of(year, DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2)),
        };
        assert!(
            midwinter.heat_at(CENTER + IVec2::new(PATCH_RADIUS, 0)) < 0.0,
            "a hauler at the treeline must feel it in deep winter"
        );
        assert!(
            midwinter.heat_at(CENTER) > 0.0,
            "the square itself still holds"
        );
    }

    #[test]
    fn a_boiler_gives_the_cold_somewhere_to_push_back() {
        let year = SEVERITY_FULL_YEAR;
        let ambient = ambient_at(day_of(year, DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2));
        let reach = |upgrades| {
            let air = Air {
                output: generator_output(FULL_BURN_FUEL, upgrades),
                ambient,
            };
            (0..=R)
                .filter(|d| air.heat_at(CENTER + IVec2::new(*d, 0)) > 0.0)
                .count()
        };
        assert!(
            reach(1) > reach(0),
            "a boiler must buy real ground back in the winter that votes for it"
        );
    }

    #[test]
    fn the_first_winter_is_gentler_than_the_ones_that_follow() {
        assert!(severity(1) < severity(2));
        assert!(severity(2) < severity(SEVERITY_FULL_YEAR));
        assert!(severity(1) > 0.0, "a mild winter is still a winter");
    }

    #[test]
    fn severity_stops_once_it_reaches_full_depth() {
        assert_eq!(severity(SEVERITY_FULL_YEAR), 1.0);
        assert_eq!(severity(SEVERITY_FULL_YEAR + 10), 1.0);
    }

    #[test]
    fn a_gentler_first_winter_is_warmer_than_a_later_one() {
        let day = DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2;
        assert!(ambient_at(day_of(1, day)) > ambient_at(day_of(SEVERITY_FULL_YEAR, day)));
    }

    #[test]
    fn summers_do_not_get_worse_with_the_years() {
        let day = DAYS_PER_SEASON + DAYS_PER_SEASON / 2;
        assert_eq!(
            ambient_at(day_of(1, day)),
            ambient_at(day_of(SEVERITY_FULL_YEAR, day)),
            "the ramp is on the winters, not the whole year"
        );
    }

    #[test]
    fn three_seasons_grow_and_winter_does_not() {
        assert!(is_growing_season(Season::Spring));
        assert!(is_growing_season(Season::Summer));
        assert!(is_growing_season(Season::Autumn));
        assert!(!is_growing_season(Season::Winter));
    }

    #[test]
    fn a_patch_grows_back_towards_its_cap_and_stops_there() {
        for kind in [Cargo::Wood, Cargo::Food] {
            let cap = 10;
            assert_eq!(
                regrowth_step(0, cap, kind, false),
                0,
                "nothing grows in winter"
            );
            let grown = regrowth_step(0, cap, kind, true);
            assert!(grown > 0, "a stripped patch comes back");
            assert_eq!(
                regrowth_step(cap, cap, kind, true),
                cap,
                "and stops at its cap"
            );
            assert_eq!(
                regrowth_step(cap - 1, cap, kind, true),
                cap,
                "without overshooting it"
            );
        }
    }

    #[test]
    fn a_year_is_its_seasons_worth_of_days() {
        assert_eq!(days_per_year(), DAYS_PER_SEASON * SEASONS_PER_YEAR);
    }

    #[test]
    fn the_band_never_grows_wider_than_the_decision_it_damps() {
        for huts in 0..8 {
            for population in [1usize, 5, 30, 100] {
                assert!(
                    haul_switch_margin(population, huts) <= HAUL_SWITCH_MAX,
                    "a band wider than a store's target is a latch, not a deadband"
                );
            }
        }
    }

    #[test]
    fn an_empty_granary_pulls_haulers_over_however_many_huts_stand() {
        let population = 30;
        let starving = supply(40, 0, population, 5);
        assert_eq!(
            haul_choice(Cargo::Wood, starving),
            Cargo::Food,
            "nobody may walk past a full hunting ground while the granary is empty"
        );
    }

    #[test]
    fn a_hauler_leaves_a_store_alone_that_the_colony_has_enough_of() {
        let patches = Patches(vec![
            Patch {
                pos: CENTER + IVec2::new(4, 0),
                kind: Cargo::Wood,
                amount: 0,
                cap: 0,
            },
            Patch {
                pos: CENTER + IVec2::new(0, 4),
                kind: Cargo::Food,
                amount: 5,
                cap: 5,
            },
        ]);
        assert!(gather_source(&patches, Cargo::Wood, CENTER, true).is_some());
        assert_eq!(
            gather_source(&patches, Cargo::Wood, CENTER, false),
            None,
            "with the granary already over target, going out for more buys nothing"
        );
    }

    #[test]
    fn a_colony_already_carrying_its_share_of_children_stops_having_them() {
        let population = 40;
        let comfortable = (population as f32 * MAX_DEPENDENT_SHARE) as usize;
        assert!(has_hands_to_spare(comfortable, population));
        assert!(
            !has_hands_to_spare(comfortable + 1, population),
            "past the share the next child is paid for out of everyone's ration"
        );
        assert!(has_hands_to_spare(0, population));
        assert!(
            !has_hands_to_spare(0, 0),
            "an empty colony has no hands at all"
        );
    }

    #[test]
    fn a_colony_of_any_size_may_carry_the_children_that_replace_it() {
        for population in [8usize, 20, 35, 60, 120] {
            let replacement = (population as f32 * REPLACEMENT_DEPENDENT_SHARE).round() as usize;
            assert!(
                has_hands_to_spare(replacement, population),
                "a colony of {population} cannot carry even the {replacement} children that \
                 would replace it, so it is dying on a schedule"
            );
        }
    }

    #[test]
    fn a_colony_carrying_more_than_the_slack_allows_stops_having_children() {
        let population = 100;
        let ceiling = (population as f32 * MAX_DEPENDENT_SHARE) as usize;
        assert!(has_hands_to_spare(ceiling, population));
        assert!(!has_hands_to_spare(ceiling + 2, population));
    }

    #[test]
    fn a_store_that_is_rising_or_flat_is_holding() {
        assert!(store_is_holding(1.4, 1.2), "rising");
        assert!(store_is_holding(1.2, 1.2), "flat");
        assert!(store_is_holding(0.5, 0.5), "flat and thin is still flat");
    }

    #[test]
    fn a_store_being_eaten_into_is_not_holding_unless_it_is_still_full() {
        assert!(
            !store_is_holding(0.8, 1.4),
            "falling and already below what the colony wants per head"
        );
        assert!(
            store_is_holding(1.05, 1.4),
            "falling from a great height but still at target is not a shortage"
        );
    }

    #[test]
    fn a_colony_with_no_season_behind_it_is_free_to_grow() {
        assert!(
            stores_are_holding([0.1; CARGO_COUNT], None),
            "a colony in its first season has nothing to compare against"
        );
    }

    #[test]
    fn one_store_failing_is_enough_to_close_the_gate() {
        let then = Some([1.5, 1.5]);
        assert!(stores_are_holding([1.6, 1.6], then));
        assert!(
            !stores_are_holding([0.4, 1.6], then),
            "wood alone can close it"
        );
        assert!(!stores_are_holding([1.6, 0.4], then), "and so can game");
    }

    #[test]
    fn the_trend_has_nothing_to_say_until_a_season_has_gone_by() {
        let mut trend = Trend::default();
        for day in 0..SEASON_DAYS - 1 {
            assert_eq!(trend.a_season_ago(), None, "only {day} days in");
            trend.record([1.0; CARGO_COUNT]);
        }
        trend.record([2.0; CARGO_COUNT]);
        assert_eq!(
            trend.a_season_ago(),
            Some([1.0; CARGO_COUNT]),
            "and then it reports the day a season back, not the latest one"
        );
    }

    #[test]
    fn the_trend_ring_keeps_walking_forward() {
        let mut trend = Trend::default();
        for day in 0..SEASON_DAYS * 3 {
            trend.record([day as f32; CARGO_COUNT]);
        }
        let oldest = (SEASON_DAYS * 3 - SEASON_DAYS) as f32;
        assert_eq!(trend.a_season_ago(), Some([oldest; CARGO_COUNT]));
    }

    #[test]
    fn the_colony_that_died_in_autumn_would_have_stopped_breeding_first() {
        // Straight off the run that died in year one: a season before the
        // collapse it held 64 fuel over 35 people, and by the time autumn bit
        // it held 44 over 40.
        let comfortable = [
            stock_share(64, FUEL_PER_CITIZEN, 35),
            stock_share(31, FOOD_PER_CITIZEN, 35),
        ];
        let sliding = [
            stock_share(44, FUEL_PER_CITIZEN, 40),
            stock_share(31, FOOD_PER_CITIZEN, 40),
        ];
        assert!(
            !stores_are_holding(sliding, Some(comfortable)),
            "a colony whose fuel per head fell through its target must stop growing"
        );
    }

    /// A citizen with `burden` owed on `kind` and nothing else the matter.
    fn owed(kind: NeedKind, burden: f32) -> Needs {
        let mut needs = Needs::newcomer();
        needs.needs[kind as usize].burden = burden;
        needs
    }

    #[test]
    fn a_need_answered_on_schedule_costs_nothing_at_all() {
        let kind = NeedKind::Rest;
        let mut need = need_at(kind.rules().high);
        need.burden = 0.0;
        // A full working cycle: down to the low mark, then slept off.
        for tick in 0..200 {
            need = need_step(need, kind, need.level <= kind.rules().low || tick > 60, 1.0);
        }
        assert_eq!(
            need.burden, 0.0,
            "a need swinging through its own band on schedule is control working"
        );
    }

    #[test]
    fn a_need_left_unanswered_starts_costing_once_it_is_past_acting_on() {
        let kind = NeedKind::Food;
        let mut need = need_at(kind.rules().low);
        need.burden = 0.0;
        for _ in 0..100 {
            need = need_step(need, kind, false, 1.0);
        }
        assert!(need.burden > 0.0, "a hunger nobody answers is what costs");
        let owed = need.burden;
        for _ in 0..500 {
            need = need_step(need, kind, true, 1.0);
        }
        assert_eq!(
            need.burden, owed,
            "and being fed later does not pay it back"
        );
    }

    #[test]
    fn survival_outranks_efficiency_however_long_the_walk_was() {
        let mut needs = owed(NeedKind::Food, 0.5);
        for _ in 0..5000 {
            needs.spend(true);
        }
        assert!(needs.detour_burden() > needs.get(NeedKind::Food).burden);
        assert_eq!(
            vote_of(&needs),
            Some(building_for(NeedKind::Food)),
            "the two tiers are never weighed against each other"
        );
    }

    #[test]
    fn a_citizen_the_colony_is_holding_up_votes_on_what_wastes_its_day() {
        let mut needs = Needs::newcomer();
        for _ in 0..100 {
            needs.spend(true);
        }
        assert_eq!(vote_of(&needs), Some(Building::GeneratorUpgrade));
    }

    #[test]
    fn a_citizen_with_nothing_the_matter_abstains() {
        assert_eq!(
            vote_of(&Needs::newcomer()),
            None,
            "contentment is a legal ballot"
        );
    }

    #[test]
    fn an_abstaining_colony_leaves_the_docket_empty() {
        let quiet = [Needs::newcomer(), Needs::newcomer()];
        assert_eq!(
            tally_votes(&quiet, &Mayor::default()),
            [0.0; BUILDING_COUNT]
        );
    }

    #[test]
    fn equally_costly_needs_still_break_in_table_order() {
        let mut needs = owed(NeedKind::Rest, 42.0);
        needs.needs[NeedKind::Food as usize].burden = 42.0;
        assert_eq!(vote_of(&needs), Some(building_for(NeedKind::Rest)));
    }

    #[test]
    fn the_ballot_forgets_both_the_needs_and_the_hours() {
        let mut needs = owed(NeedKind::Food, 500.0);
        for _ in 0..100 {
            needs.spend(true);
        }
        needs.forget_before_ballot();
        for kind in NEEDS {
            assert_eq!(needs.get(kind).burden, 0.0, "{kind:?} still owed");
        }
        assert_eq!(needs.detour_burden(), 0.0);
        assert_eq!(vote_of(&needs), None);
    }

    #[test]
    fn a_bigger_colony_spends_more_every_tick() {
        assert!(demand_per_tick(60, 0) > demand_per_tick(30, 0));
        assert!(
            demand_per_tick(30, 1) > demand_per_tick(30, 0),
            "a boiler eats too"
        );
        assert!(
            demand_per_tick(0, 0) > 0.0,
            "the fire burns for an empty colony"
        );
    }

    #[test]
    fn a_colony_that_cannot_fetch_what_it_spends_takes_on_nobody() {
        let population = 40;
        let barely = demand_per_tick(population + 1, 0) / 30.0;
        assert!(
            !can_afford_a_mouth(30, barely, population, 0),
            "breaking even is not enough to carry a winter"
        );
        assert!(can_afford_a_mouth(
            30,
            barely * (1.0 + WINTER_MARGIN) * 1.01,
            population,
            0
        ));
    }

    #[test]
    fn more_hands_buy_more_mouths() {
        let per_hand = 0.03;
        assert!(can_afford_a_mouth(60, per_hand, 20, 0));
        assert!(!can_afford_a_mouth(5, per_hand, 20, 0));
    }

    #[test]
    fn the_flow_has_nothing_to_report_until_a_day_has_closed() {
        let mut flow = Flow::default();
        flow.delivered(10);
        assert_eq!(flow.per_hand(), None, "a day still open says nothing");
        flow.close_the_day(0);
        assert_eq!(
            flow.per_hand(),
            None,
            "and neither does a day with no hands"
        );
        flow.delivered(24);
        flow.close_the_day(1);
        assert!(flow.per_hand().is_some());
    }

    #[test]
    fn one_hauler_bringing_a_day_of_units_reads_as_one_a_tick() {
        let mut flow = Flow::default();
        flow.delivered(ticks_per_day() as u32);
        flow.close_the_day(1);
        assert_eq!(flow.per_hand(), Some(1.0));
    }

    #[test]
    fn the_flow_averages_over_the_days_it_holds() {
        let mut flow = Flow::default();
        for _ in 0..SEASON_DAYS * 2 {
            flow.delivered(ticks_per_day() as u32 * 2);
            flow.close_the_day(2);
        }
        assert_eq!(flow.per_hand(), Some(1.0), "two hands, twice the units");
    }

    #[test]
    fn the_colony_that_crashed_would_have_stopped_taking_on_mouths() {
        // From the run that died in autumn of year one: thirty hands fetching
        // about nine hundredths of a unit a tick between them, at the forty
        // mouths it had reached by the time the cold caught it.
        let hands = 30;
        let per_hand = 0.9 / hands as f32;
        assert!(
            !can_afford_a_mouth(hands, per_hand, 40, 0),
            "thirty pairs of hands cannot carry a forty-first mouth through a winter"
        );
        assert!(
            can_afford_a_mouth(hands, per_hand, 30, 0),
            "but the colony it started as was well within itself"
        );
    }

    #[test]
    fn the_milestones_are_named_in_order_and_end_at_adulthood() {
        let mut previous = 0.0;
        for (stage, age) in MILESTONE_AGES.into_iter().enumerate() {
            assert!(
                age > previous,
                "milestone {stage} does not come after the last"
            );
            previous = age;
        }
    }

    #[test]
    fn the_early_years_weigh_heaviest() {
        let mut previous = f32::MAX;
        for (stage, weight) in MILESTONE_WEIGHTS.into_iter().enumerate() {
            assert!(
                weight < previous,
                "stage {stage} weighs at least as much as the one before it"
            );
            previous = weight;
        }
    }

    #[test]
    fn a_stage_of_plenty_raises_a_stat_and_a_stage_of_want_lowers_it() {
        let middling = 0.5;
        assert!(milestone_step(middling, 1.0, MILESTONE_WEIGHTS[0]) > middling);
        assert!(milestone_step(middling, 0.0, MILESTONE_WEIGHTS[0]) < middling);
        assert_eq!(
            milestone_step(middling, FORMATION_NEUTRAL, MILESTONE_WEIGHTS[0]),
            middling,
            "a childhood exactly as good as expected changes nothing"
        );
    }

    #[test]
    fn the_same_good_year_is_worth_more_to_a_child_who_already_has_something() {
        let poor = 0.3;
        let rich = 0.7;
        let gained = |stock: f32| milestone_step(stock, 1.0, MILESTONE_WEIGHTS[1]) - stock;
        assert!(
            gained(rich) > gained(poor),
            "investment complements the stock it lands on, it does not substitute for it"
        );
    }

    #[test]
    fn a_bad_start_cannot_be_bought_back_at_full_price() {
        let steady = 0.5;
        let starved_then_fed = {
            let after_famine = milestone_step(steady, 0.0, MILESTONE_WEIGHTS[0]);
            milestone_step(after_famine, 1.0, MILESTONE_WEIGHTS[1])
        };
        let fed_then_starved = {
            let after_plenty = milestone_step(steady, 1.0, MILESTONE_WEIGHTS[0]);
            milestone_step(after_plenty, 0.0, MILESTONE_WEIGHTS[1])
        };
        assert!(
            starved_then_fed < fed_then_starved,
            "the same two years in the other order do not come out the same"
        );
    }

    #[test]
    fn a_stat_never_leaves_its_range() {
        let mut stock = 0.5;
        for _ in 0..50 {
            stock = milestone_step(stock, 1.0, MILESTONE_WEIGHTS[0]);
        }
        assert!(stock <= STAT_MAX);
        let mut stock = 0.5;
        for _ in 0..50 {
            stock = milestone_step(stock, 0.0, MILESTONE_WEIGHTS[0]);
        }
        assert!(stock >= STAT_MIN);
    }

    #[test]
    fn each_stat_is_raised_on_what_actually_feeds_it() {
        assert_eq!(
            provision_for(Stat::Strength, 0.2, 0.9),
            0.9,
            "the body eats"
        );
        assert_eq!(
            provision_for(Stat::Hardiness, 0.2, 0.9),
            0.2,
            "and is warmed"
        );
        let both = provision_for(Stat::Wits, 0.2, 0.9);
        assert!(
            both > 0.2 && both < 0.9,
            "wits take whatever the household had of either"
        );
    }

    #[test]
    fn the_body_keeps_growing_after_adulthood_and_the_mind_does_not() {
        assert!(catches_up(Stat::Strength));
        assert!(catches_up(Stat::Hardiness));
        assert!(
            !catches_up(Stat::Wits),
            "the physical deficit partly recovers and the cognitive one does not"
        );
    }

    #[test]
    fn a_founder_is_the_residual_and_nothing_else() {
        let a = Stats::migrant(1);
        let b = Stats::migrant(2);
        for stat in STATS {
            assert!(a.of(stat) >= STAT_MIN && a.of(stat) <= STAT_MAX);
        }
        assert!(
            STATS.into_iter().any(|stat| a.of(stat) != b.of(stat)),
            "two people who walked in out of the cold are not the same person"
        );
    }

    #[test]
    fn two_children_of_one_childhood_still_differ() {
        let raised = |seed| {
            let mut childhood = Upbringing::born(seed);
            for age in MILESTONE_AGES {
                childhood.resolve(age, 0.8, 0.8);
            }
            childhood.stats()
        };
        let one = raised(11);
        let other = raised(12);
        assert!(
            STATS.into_iter().any(|stat| one.of(stat) != other.of(stat)),
            "the residual is what keeps siblings from being clones"
        );
    }

    #[test]
    fn a_childhood_of_plenty_beats_a_childhood_of_want() {
        let raised = |provision: f32| {
            let mut childhood = Upbringing::born(7);
            for age in MILESTONE_AGES {
                childhood.resolve(age, provision, provision);
            }
            childhood.stats()
        };
        let fat = raised(1.0);
        let lean = raised(0.0);
        for stat in STATS {
            assert!(
                fat.of(stat) > lean.of(stat),
                "{stat:?} did not come out ahead on a childhood twice as good"
            );
        }
    }

    #[test]
    fn a_milestone_is_only_ever_resolved_once() {
        let mut childhood = Upbringing::born(3);
        childhood.resolve(MILESTONE_AGES[0], 1.0, 1.0);
        let after_one = childhood.stats();
        childhood.resolve(MILESTONE_AGES[0] - 0.1, 1.0, 1.0);
        assert_eq!(
            childhood.stats().of(Stat::Wits),
            after_one.of(Stat::Wits),
            "a birthday does not come round twice"
        );
    }

    #[test]
    fn the_middle_of_a_colony_is_the_middle_of_what_it_has() {
        assert_eq!(
            median(&mut []),
            FORMATION_NEUTRAL,
            "an empty colony is unremarkable"
        );
        assert_eq!(median(&mut [0.4]), 0.4);
        assert_eq!(
            median(&mut [0.9, 0.1, 0.5]),
            0.5,
            "and order does not matter"
        );
    }

    #[test]
    fn a_good_adolescence_buys_back_the_body_and_not_the_mind() {
        let starved = || {
            let mut childhood = Upbringing::born(5);
            for age in MILESTONE_AGES {
                childhood.resolve(age, 0.0, 0.0);
            }
            childhood
        };
        let before = starved().stats();
        let mut repaired = starved();
        for _ in 0..(ticks_per_year() * 3) {
            repaired.catch_up(ADULT_AGE + 1.0, 1.0, 1.0);
        }
        let after = repaired.stats();
        assert!(after.of(Stat::Strength) > before.of(Stat::Strength));
        assert!(after.of(Stat::Hardiness) > before.of(Stat::Hardiness));
        assert_eq!(
            after.of(Stat::Wits),
            before.of(Stat::Wits),
            "no amount of later plenty reopens a mind that closed at adulthood"
        );
    }

    #[test]
    fn the_catch_up_runs_out_with_the_years() {
        let gained = |age: f32| {
            let mut childhood = Upbringing::born(5);
            for age in MILESTONE_AGES {
                childhood.resolve(age, 0.0, 0.0);
            }
            let before = childhood.stats().of(Stat::Strength);
            for _ in 0..ticks_per_year() {
                childhood.catch_up(age, 1.0, 1.0);
            }
            childhood.stats().of(Stat::Strength) - before
        };
        assert!(gained(ADULT_AGE + 1.0) > gained(CATCHUP_UNTIL - 1.0));
        assert_eq!(gained(CATCHUP_UNTIL), 0.0, "and then it is over");
    }
}
