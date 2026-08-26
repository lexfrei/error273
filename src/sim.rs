use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

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

pub const fn per_season(rate: f32) -> f32 {
    rate / ticks_per_season() as f32
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
/// How far the air moves between the coldest hour of a day and the warmest.
/// Small against the year's swing on purpose: a day is a decision about when to
/// go out, and a season is a decision about whether to.
pub const DIURNAL_SWING: f32 = 6.0;
/// When the cold is worst. Before dawn rather than at midnight, which is where
/// it falls outside a window as well.
pub const COLDEST_HOUR: u64 = 5;
/// Weather, as against climate: how far today has wandered from the season it
/// belongs to, and how long that wandering lasts.
///
/// The shape is the one stochastic weather generators use for temperature --
/// today's departure from the seasonal mean is a fraction of yesterday's plus a
/// fresh push, which is a first-order autoregression. What is borrowed is the
/// shape; the numbers are ours, because a degree here is a warmth a citizen
/// loses and not a degree Celsius.
pub const FRONT_DAYS: f32 = 6.0;
/// How hard one day can push, and how far a front may ever take the air from
/// the season underneath it.
pub const FRONT_STEP: f32 = 2.5;
pub const FRONT_CAP: f32 = 8.0;
pub const FRONT_SALT: u64 = 0x71;
pub const SPELL_SALT: u64 = 0x72;

/// What share of itself a colony has to bury in one winter before the survivors
/// stop measuring against the year they had, how much of a bad thing they feel
/// while that lasts, and how long it lasts.
///
/// A colony that has just buried a quarter of itself is not hurt by cold and
/// hunger in the measure it was a season ago, because it is no longer measuring
/// against the same year; the same circumstances weigh less, survivors need
/// less to stay level, and it wears off.
///
/// It moves nothing a citizen acts on. Lowering the mark somebody sets out for
/// warmth at would send them out later and kill them; this is about what a day
/// costs them to live through, which is a different question with a different
/// reader.
pub const LOSS_SHARE: f32 = 0.25;
pub const EXPECTATIONS_AFTER_LOSS: f32 = 0.6;
pub const EXPECTATIONS_SEASONS: u64 = 4;

/// The most the weather may ever take off the air in one spell.
///
/// This is the hurt-but-not-kill rule as a number. A budgeted event may push a
/// colony that is already marginal over the edge; it may not take a healthy one
/// out in a single blow, and the only thing standing between those two is this
/// ceiling. It is set against what the fire holds back rather than picked: the
/// generator reaches about thirteen cells on an average day and about nine at
/// the winter floor, and this takes roughly the difference again -- a spell at
/// full depth costs the colony about as much ground as deep winter already does.
pub const BUDGET_CEILING: f32 = 12.0;
/// What a colony with nothing to its name is still worth hitting with, and what
/// each head and each store-share above that buys.
pub const BUDGET_FLOOR: f32 = 2.0;
pub const BUDGET_PER_HEAD: f32 = 0.06;
pub const BUDGET_PER_SHARE: f32 = 2.0;
/// How much of a colony's stores count towards what it can be hit with. A
/// colony ten times over its target is not ten times worth hitting.
pub const BUDGET_SHARE_CAP: f32 = 3.0;

/// The grace a colony earns by not burying anybody, and loses when it does.
/// Bounds and shape from the storyteller this is taken from: it rises with time
/// since the last death and drops when one happens.
pub const ADAPT_MIN: f32 = 0.4;
pub const ADAPT_MAX: f32 = 1.5;
pub const ADAPT_RISE_PER_YEAR: f32 = 0.35;
pub const ADAPT_DEATH_COST: f32 = 0.12;

/// How long a spell takes to arrive and to leave, and how long one lasts.
///
/// The onset is the point: the air reading has to show the approach before the
/// depth lands, so that what a watcher sees is weather coming rather than an
/// aftermath. It is the same idea as stopping a fast-forward a minute before
/// the event instead of at it.
pub const SPELL_ONSET_DAYS: u64 = 3;
pub const SPELL_DAYS_MIN: u64 = 6;
pub const SPELL_DAYS_MAX: u64 = 14;
/// How often a spell begins, at a full budget. Scaled down with the budget, so
/// a colony with nothing sees weather rarely as well as shallowly.
pub const SPELL_CHANCE_PER_DAY: f32 = 0.035;
/// Where a cold snap stops being a snap. A spell deeper than this share of the
/// ceiling is called by its harder name.
pub const BLIZZARD_SHARE: f32 = 0.6;
/// How much of a spell's draw goes the warm way.
pub const THAW_SHARE: f32 = 0.3;
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
pub const WOOD_PER_CELL: u32 = 80;
pub const FOOD_PER_CELL: u32 = 90;

// What the colony starts with. These belong beside the rates they are tuned
// against, not in the app wiring.
pub const START_FUEL: u32 = 20;
pub const START_FOOD: u32 = 60;
pub const START_RING: f32 = 2.0;

pub const NEED_MAX: f32 = 100.0;
pub const NEED_COUNT: usize = 4;
/// The warmth a citizen keeps in hand on top of the walk home. It is set to the
/// mark warmth used to act on, so a citizen standing at the fire behaves exactly
/// as they always did and only distance changes anything.
pub const CAUTION_BASE: f32 = 25.0;
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
/// A waystation is a brazier and a woodshed, not a boiler: cheaper to raise
/// than the fire it stands in for, and dear to keep.
pub const WAYSTATION_WOOD_COST: u32 = 18;
// How much further a boiler pushes the generator's ceiling once the woodpile
// can keep it fed.
pub const UPGRADE_HEAT: f32 = 18.0;
/// What a lit waystation puts out. It buys nothing at the hearth and the whole
/// of its own distance for whoever is working out there.
pub const WAYSTATION_HEAT: f32 = 40.0;
/// What it holds, and what it burns. A small cache on purpose: a chain of them
/// is meant to be a standing tax on the workforce, not a thing filled once.
pub const WAYSTATION_CACHE: u32 = 12;
pub const WAYSTATION_BURN_EVERY: u64 = 6 * TICKS_PER_HOUR;
/// What a slab costs to raise, and how long the colony waits before it gives
/// up on somebody. A year is the wait: long enough that a scout still out is
/// not written off, short enough that the colony is not counting the same
/// absence a decade later.
pub const MEMORIAL_WOOD_COST: u32 = 8;
pub const MEMORIAL_AFTER: u64 = ticks_per_year();

/// How far past the warmth it already has the colony may put a new fire.
///
/// A post is stocked by haulers walking out to it, and a hauler can only go as
/// far as their own warmth carries them there and back. So a chain cannot be
/// laid in one reach: it grows a link at a time, each post standing close
/// enough to the last that somebody can supply it, and this is the length of a
/// link.
pub const WAYSTATION_STEP: f32 = (NEED_MAX - CAUTION_BASE) / 2.0;
/// How far out a boiler can still hold the freezing line. A citizen whose walk
/// back starts further out than a bigger fire can ever reach is not asking for
/// a bigger fire; they are asking for a nearer one.
pub const BOILER_REACHES: i32 =
    ((GENERATOR_HEAT + UPGRADE_HEAT + AMBIENT_MEAN) / HEAT_FALLOFF) as i32;
pub const BUILDING_COUNT: usize = 4;
pub const CARGO_COUNT: usize = 2;
pub const SEASON_DAYS: usize = DAYS_PER_SEASON as usize;

// The world has no edge. It is generated in squares, each one a pure function
// of the world seed and its own coordinates, so a chunk may be realised in any
// order and a run still replays.
pub const CHUNK: i32 = 64;
/// Cells to a step of the lattice the generated field is interpolated over.
pub const LATTICE: i32 = 24;
/// How far out richness stops rising. It is not where a trip is worth most:
/// what a citizen can carry is set by the citizen, so a longer walk brings home
/// no more in one go. It is where ground stops giving up more before it is bare,
/// which is as far out as there is any reason to walk. Inside the old rim
/// richness is exactly what it always was, which keeps the early game unretuned.
pub const RICHNESS_BEST: i32 = PATCH_RADIUS + 160;
/// How many cells of walking each step of richness costs to reach.
pub const RICHNESS_RUN: i32 = 80;
pub const RICHNESS_CAP: f32 = (RICHNESS_BEST - PATCH_RADIUS) as f32 / RICHNESS_RUN as f32;
/// Patches to a chunk, set from the density the founding rings ran at -- twelve
/// of them inside a disc of radius seventeen, which is about one patch in every
/// seventy-six cells -- so the early game is not quietly retuned by the world
/// changing shape underneath it.
pub const PATCHES_PER_CHUNK: usize = 54;
pub const WOOD_SHARE: f32 = 2.0 / 3.0;
/// How far the generated field is allowed to move a patch's worth either way.
pub const FIELD_SWING: f32 = 0.5;
pub const CHUNK_SALT: u64 = 0x61;
pub const SCOUT_TURN_SALT: u64 = 0x63;
pub const SCOUT_PICK_SALT: u64 = 0x64;
pub const FIELD_SALT: u64 = 0x62;
/// As far as anybody walks for work. Past here the ground stops getting better
/// for the walk, so there is nothing out there worth the extra cold.
pub const SEARCH_LIMIT: i32 = RICHNESS_BEST;
/// The shortest leg a scout draws, and the longest. The tail between them goes
/// as one over the square of the distance, which is the search-theory answer for
/// sparse targets that are worth revisiting; the cap is where the ground stops
/// getting better, so nothing is drawn that nobody would walk.
pub const SCOUT_STEP_MIN: i32 = 8;
pub const SCOUT_STEP_MAX: i32 = SEARCH_LIMIT;
/// How many of the colony's hands may be out looking rather than working.
pub const SCOUT_SHARE: f32 = 0.04;

/// How far out the colony's day-to-day work actually happens. Nothing enforces
/// it -- a citizen may walk to the limit -- but it is where the ground the
/// colony lives off stands, so it is what the status columns total and what the
/// window frames.
pub const NEAR_GROUND: i32 = 24;
/// How far around a citizen the world is kept in memory between seasons.
pub const FORGET_BEYOND: i32 = 64;
/// How much of the world the window shows around the hearth.
pub const VIEW_RADIUS: i32 = NEAR_GROUND;

/// Whether a cell is one the window draws. The world has no edge, so this is a
/// statement about the frame and never about where anybody may walk.
pub fn on_frame(cell: IVec2) -> bool {
    (cell - CENTER).abs().max_element() <= VIEW_RADIUS
}

/// What a walk counts as when there is no work of that kind anywhere in reach.
/// It is the longest walk there is, so a spare hand who found nothing never
/// scores better than one who found work at the end of the world.
pub const WALK_UNFOUND: i32 = SEARCH_LIMIT;
/// The seed the world is generated from. One number, fixed, so a run replays.
pub const WORLD_SEED: u64 = 0x2026;
/// The most of the world the colony may be holding at once, in cells. It is a
/// ceiling on memory, not on where anybody can walk.
///
/// The two halves it covers grow for different reasons, which is why it has to
/// cover both. Realised chunks follow where the colony is standing: it draws
/// them as it goes and drops the ones behind it. Remembered cuts cannot be
/// dropped -- letting one go is letting a stripped treeline come back full --
/// so they follow how much ground the colony has ever worked, and that is the
/// half a long run would break the ceiling with.
#[cfg(not(feature = "window"))]
pub const WORLD_CELLS_HELD: usize = 1 << 20;

/// Whether that ceiling is being kept. The headless build checks it every tick,
/// which is the only place it is checked: it is an instrument reading, and a
/// simulation that policed its own memory would be able to hide a leak by
/// quietly forgetting more.
#[cfg(not(feature = "window"))]
pub fn world_is_bounded(cells_held: usize) -> bool {
    cells_held <= WORLD_CELLS_HELD
}

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
// A body hardens to cold with exposure and loses it again without it. The
// diving women of Korea and Japan ran a basal metabolic rate about 5% above
// standard in summer against 35% in winter, and the seasonal swing vanished
// entirely once wetsuits cut the exposure. Gained over a season of working the
// cold, lost over three of not.
pub const ACCLIMATION_WORTH: f32 = 0.15;
pub const ACCLIMATION_GAIN: f32 = per_season(1.0);
pub const ACCLIMATION_LOSS: f32 = per_season(1.0 / 3.0);
// How much of a span the colony cannot account for, at the lean end and the fat
// end. Scarr-Rowe, taken as a stylised rule rather than a transferred number:
// shared environment explained roughly sixty per cent of the variance in
// impoverished families and near zero in affluent ones, so the direction is the
// finding and the endpoints are ours.
pub const RESIDUAL_SHARE_LEAN: f32 = 0.2;
pub const RESIDUAL_SHARE_FAT: f32 = 0.8;
/// How far a childhood can move a span either way.
pub const LIFESPAN_RAISED_SWING: f32 = 0.2;

// Trades. What a citizen is worth at work is what the colony raised in them,
// what they have practised, and -- once the mood layer exists -- how well they
// are holding up.
pub const TRADE_COUNT: usize = 4;
/// What a performance is made of. The terms are named because the whole point
/// of this layer is that a role changes an outcome and can be shown to: a
/// watcher should be able to read why a night went the way it did.
///
/// The tradition term is a seam and is always nothing today. When a colony has
/// traditions, a performance that matches one is worth more than one that does
/// not, and this is where that arrives.
pub const QUALITY_AUDIENCE: f32 = 0.3;
pub const QUALITY_WARMTH: f32 = 0.2;
pub const QUALITY_PERFORMER: f32 = 0.4;
pub const QUALITY_TRADITION: f32 = 0.1;
/// The audience past which another body adds nothing, and the warmth at which
/// a place is as good a stage as it is going to be.
pub const AUDIENCE_FULL: f32 = 8.0;
pub const QUALITY_WARM_ENOUGH: f32 = 10.0;
/// How long a night that went well is still worth something. A night that did
/// not is worth nothing rather than worth something bad: the boredom it failed
/// to lift is already the penalty.
pub const CHEER_DAYS_FUN: u64 = 3;
pub const CHEER_DAYS_UNFORGETTABLE: u64 = 6;
pub const PERFORMANCE_SALT: u64 = 0x81;
/// How far a performance carries, and the hour of the evening it happens at.
/// Once a day, because a performance is an event a watcher can point at rather
/// than a hum in the background.
pub const PERFORMANCE_REACH: i32 = 4;
pub const PERFORMANCE_HOUR: u64 = 19;
/// How long before it starts the evening is called, in hours. A citizen walks a
/// cell an hour, so the same number is how far away anybody will come from: the
/// call is the catchment, and it needs no second constant to say so.
pub const PERFORMANCE_CALL: u64 = 8;
pub const FIT_FLOOR: f32 = 0.5;
pub const FIT_STAT: f32 = 0.6;
pub const FIT_PRACTICE: f32 = 0.5;
pub const FIT_CEILING: f32 = 1.8;
/// How heavily a need weighs when the colony works out how distracted somebody
/// is. Warmth, rest and hunger are the top tier together; recreation and social
/// enter below it, which is what keeps them from competing with survival in a
/// crisis rather than needing a rule that says they must not.
pub const NEED_WEIGHT_SURVIVAL: f32 = 10.0;
/// What a need weighs that nobody dies of. Low enough that it can never take a
/// citizen away from a survival need however far it has fallen, which the
/// ordering reads off this number rather than off a rule about recreation.
pub const NEED_WEIGHT_COMFORT: f32 = 2.0;
/// The worst focus a citizen can be reduced to, and where the amplified part of
/// the cost starts biting. Half is the floor because that is the penalty the
/// shape this is taken from tops out at; the knee is near the bottom because a
/// need at a tenth is a different thing from a need at half.
pub const FOCUS_FLOOR: f32 = 0.5;
/// Where a mood sits with nothing being felt either way, and the top of the
/// scale it is printed on.
pub const MOOD_BASE: f32 = 50.0;
pub const MOOD_MAX: f32 = 100.0;
/// How far a mood travels towards its target in a day, up and down. Up is
/// quicker, which is the shape of the bar this is taken from.
pub const MOOD_RISE_PER_DAY: f32 = 12.0;
pub const MOOD_FALL_PER_DAY: f32 = 8.0;
/// How far a mood has to fall before the days start leaving a mark, and how far
/// it has to climb before one starts to fade. Two marks and not one, per ADR
/// 0003: the gap between them is what stops a citizen's history flickering with
/// their afternoon.
/// Left where it was, and deliberately. Swept over four values on six worlds
/// against a world that can hurt: at 30 and 40 nobody is ever marked, and at 50
/// and 60 everybody is, up to sixty-one at once. There is no value between that
/// gives the rare mark this is for, because the colony's misery is brief rather
/// than sustained -- a mood sits near fifty and dips to twenty for days, and
/// this is built to record years. What it is waiting for is a mechanic that
/// produces prolonged hardship rather than sharp shocks, and moving the mark
/// would not produce one.
pub const HARDSHIP_MARK: f32 = 30.0;
pub const HARDSHIP_EASE: f32 = 70.0;
pub const HARDSHIP_MAX: f32 = 100.0;
/// How deep a mark gets in a year of misery, and how much of one a year of
/// contentment takes back. The second is far smaller on purpose: a bar that
/// recovers as fast as it fills is the mood over again, and what this is for is
/// the part of somebody that the mood has already stopped explaining.
pub const HARDSHIP_GAIN_PER_YEAR: f32 = 40.0;
pub const HARDSHIP_FADE_PER_YEAR: f32 = 8.0;
pub const FOCUS_KNEE: f32 = 15.0;
/// How much heavier a point of a need costs below the knee than above it.
///
/// Chosen against a world that can hurt, which is what it was waiting for, and
/// chosen on the instrument rather than on the arc: over twelve worlds the arc
/// cannot tell 0.5, 1.0 and 1.5 apart -- three live seeds and the wins split --
/// while the number itself is three times livelier at 0.5, visibly below full
/// focus on 264 days across those worlds against 92 at 1.0.
///
/// It runs the way round it does because the bite is in the normalisation as
/// well as in the term: a heavier bite raises what the worst possible case
/// costs, which makes every ordinary shortfall a smaller share of it. Trading
/// away the ordinary range to sharpen the extreme is the wrong trade for a
/// number whose whole purpose is to explain an underperformance before it
/// becomes a collapse.
pub const FOCUS_KNEE_BITE: f32 = 0.5;
/// Where the bands fall. Display only: the value underneath is continuous, and
/// nothing in the simulation reads a band.
pub const FOCUS_UNFOCUSED: f32 = 0.95;
pub const FOCUS_DISTRACTED: f32 = 0.85;
pub const FOCUS_BADLY: f32 = 0.7;
/// What anybody carries, before what they are is counted. One, because a load
/// is taken from the patch at the moment it is picked up and a trip that lifted
/// nothing would still have stripped the treeline.
pub const HAUL_BASE: f32 = 1.0;
pub const HAUL_LOAD_SWING: f32 = 1.0;
// Distance is the Chebyshev norm, which is the true travel time under
// one-cell-per-tick king moves, and is deliberately the wrong distance for
// anything but that.
pub const WEIGHT_DISTANCE: f32 = 0.05;
pub const WEIGHT_EXPERIENCE: f32 = 1.0;
pub const WEIGHT_BIAS: f32 = 0.5;
/// Filled from the best few rather than the best, which buys variety with no
/// randomness in the model.
pub const ASSIGNMENT_SHORTLIST: usize = 3;
pub const EXPERIENCE_GAIN: f32 = per_season(1.0);
pub const EXPERIENCE_RUST: f32 = per_season(0.2);
// A trade nobody could keep would not be a trade, checked where it cannot be
// skipped.
const _: () = assert!(EXPERIENCE_GAIN > EXPERIENCE_RUST);
/// A fifth to a quarter of the workforce held back for work no trade covers.
pub const ENTERTAINER_SHARE: f32 = 0.04;
pub const LABORER_SHARE: f32 = 0.22;
/// What a child of the house starts with of the head's practice in it.
pub const INHERITED_SHARE: f32 = 0.4;
/// One founder in this many takes to hunting. The fire is most of what a colony
/// spends, so most of the founding party goes to the treelines.
pub const HUNTERS_ONE_IN: usize = 4;

// What the colony says about a citizen. Never a number: one word per stat,
// bucketed against the colony's own middle, so the same person is described
// differently in a different colony.
pub const REGARD_STEP: f32 = 0.08;
/// Days of watched work before the colony trusts what it has seen over what it
/// remembers of the upbringing.
pub const WORKER_DAYS_TO_KNOW: f32 = 60.0;
/// How far a childhood alone is allowed to move the guess. A band, never a
/// measurement -- the colony watched the warmth and the food, not the citizen.
pub const PRIOR_STRENGTH: f32 = 0.35;
/// How many days a store may go short before the colony stops respecting
/// trades over it. Banished shipped exactly this valve after its own players
/// found the alternative: a job nobody has taken for long enough stops being
/// somebody's job and becomes anybody's.
pub const POSTING_STALE_AFTER: f32 = 20.0;
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

/// How the colony describes one stat of one citizen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regard {
    Poor,
    Below,
    Middling,
    Above,
    Strong,
}

impl Regard {
    pub fn word(self) -> &'static str {
        match self {
            Regard::Poor => "poor",
            Regard::Below => "weak",
            Regard::Middling => "fair",
            Regard::Above => "good",
            Regard::Strong => "strong",
        }
    }
}

/// Which band a value falls in, measured against the middle of this colony
/// rather than against any fixed scale.
pub fn regard_of(value: f32, median: f32) -> Regard {
    let apart = value - median;
    if apart <= -2.0 * REGARD_STEP {
        Regard::Poor
    } else if apart <= -REGARD_STEP {
        Regard::Below
    } else if apart < REGARD_STEP {
        Regard::Middling
    } else if apart < 2.0 * REGARD_STEP {
        Regard::Above
    } else {
        Regard::Strong
    }
}

/// Whether the colony has watched somebody work long enough to trust what it
/// has seen over what it remembers of how they were raised.
pub fn is_known(worker_days: f32) -> bool {
    worker_days >= WORKER_DAYS_TO_KNOW
}

/// The colony's running estimate of a citizen: at first the band the childhood
/// it watched suggests, then the work it has actually seen, and a blend in
/// between. It starts wrong on purpose and converges.
pub fn estimate(raised: f32, prosperity: f32, worker_days: f32) -> f32 {
    let watched = (worker_days / WORKER_DAYS_TO_KNOW).clamp(0.0, 1.0);
    let guessed = FORMATION_NEUTRAL + (prosperity - FORMATION_NEUTRAL) * PRIOR_STRENGTH;
    guessed * (1.0 - watched) + raised * watched
}

/// How long each store has stood short. A colony that respects trades over a
/// store nobody is filling has made its first guess at a workforce permanent.
#[derive(Resource, Default)]
pub struct Postings {
    short_for: [f32; CARGO_COUNT],
}

impl Postings {
    pub fn note(&mut self, cargo: Cargo, short: bool) {
        self.short_for[cargo as usize] = if short {
            self.short_for[cargo as usize] + 1.0
        } else {
            0.0
        };
    }

    pub fn days_short(&self, cargo: Cargo) -> f32 {
        self.short_for[cargo as usize]
    }
}

pub fn posting_is_stale(days_short: f32) -> bool {
    days_short >= POSTING_STALE_AFTER
}

/// Who may be taken for a vacancy. The spare hands always; once a posting has
/// gone stale, anybody who is not already doing that work.
pub fn may_be_taken(trade: Trade, wanted: Trade, stale: bool) -> bool {
    trade == Trade::Laborer || (stale && trade != wanted)
}

/// What a citizen does with their days. The labourer pool is a trade like any
/// other and is what makes the others fillable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trade {
    Woodcutter,
    Hunter,
    Laborer,
    Entertainer,
}

pub const TRADES: [Trade; TRADE_COUNT] = [
    Trade::Woodcutter,
    Trade::Hunter,
    Trade::Laborer,
    Trade::Entertainer,
];

/// The stat a trade leans on. A labourer is judged on the same thing a
/// woodcutter is, because that is most of what the colony asks of them.
pub fn trade_stat(trade: Trade) -> Stat {
    match trade {
        Trade::Woodcutter | Trade::Laborer => Stat::Strength,
        Trade::Hunter | Trade::Entertainer => Stat::Wits,
    }
}

/// What a trade fetches, or nothing for a labourer, who goes wherever the
/// colony is shorter.
pub fn trade_cargo(trade: Trade) -> Option<Cargo> {
    match trade {
        Trade::Woodcutter => Some(Cargo::Wood),
        Trade::Hunter => Some(Cargo::Food),
        Trade::Laborer | Trade::Entertainer => None,
    }
}

pub fn trade_for(cargo: Cargo) -> Trade {
    match cargo {
        Cargo::Wood => Trade::Woodcutter,
        Cargo::Food => Trade::Hunter,
    }
}

/// How well a citizen fits the work: what the colony raised in them and what
/// they have actually done, both saturating so neither runs away.
pub fn trade_fit(stat: f32, experience: f32) -> f32 {
    (FIT_FLOOR + stat * FIT_STAT + experience.clamp(0.0, 1.0) * FIT_PRACTICE).min(FIT_CEILING)
}

/// What a citizen is losing to one need going unmet, weighted against the
/// others. Continuous all the way down, with the last of a need costing more
/// than the first of it, because a need at a tenth is a different thing from a
/// need at half and a penalty that switches on at a line reads as arbitrary.
///
/// It is measured from the mark a citizen starts acting at and not from the one
/// the need counts as met at, which is the same line the ballot's burden is
/// counted past and for the same reason: a need swinging through its own band
/// on schedule is control working, and charging a citizen for the ordinary
/// cycle of being warm, getting cold and going back to the fire is charging
/// them for doing their job. Measured from met, the colony died on every world
/// tried, in its seventies at best and its fourth year at worst.
impl NeedKind {
    /// Whether this is one of the needs the colony runs on, as opposed to one
    /// it is merely nicer for having met. Read off the weight rather than a
    /// list, so the two classes cannot disagree about which need is which.
    ///
    /// Everything structural asks this before it counts a need: what a citizen
    /// is worth at the work, and whether the colony is in any shape to grow.
    /// A comfort need is paid in mood and nowhere else. Letting one into those
    /// two would be a tax every colonist pays forever -- nothing a citizen does
    /// alone raises it -- levied hardest on the colony too poor to spare
    /// anybody to the evening, which is the shape this layer exists not to be.
    pub fn is_survival(self) -> bool {
        self.rules().weight >= NEED_WEIGHT_SURVIVAL
    }
}

fn focus_cost(kind: NeedKind, level: f32) -> f32 {
    let rules = kind.rules();
    let short = ((rules.low - level) / rules.low).clamp(0.0, 1.0);
    let bite = ((FOCUS_KNEE - level) / FOCUS_KNEE).clamp(0.0, 1.0);
    rules.weight * (short + bite * FOCUS_KNEE_BITE)
}

/// Something true of a citizen that they feel about.
///
/// Named rather than summed straight into a number, because a colony that can
/// only print a total cannot tell anybody why, and the list is the thing a face
/// would show. Every one of these is read off state that already exists; none
/// of them is an event this layer invented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thought {
    Cold,
    Hungry,
    Worn,
    Bored,
    WrongWork,
    Missing,
    Plenty,
    Cheered,
}

pub const THOUGHT_COUNT: usize = 8;
pub const THOUGHTS: [Thought; THOUGHT_COUNT] = [
    Thought::Cold,
    Thought::Hungry,
    Thought::Worn,
    Thought::Bored,
    Thought::WrongWork,
    Thought::Missing,
    Thought::Plenty,
    Thought::Cheered,
];

impl Thought {
    pub fn name(self) -> &'static str {
        match self {
            Thought::Cold => "cold",
            Thought::Hungry => "hungry",
            Thought::Worn => "worn out",
            Thought::Bored => "bored",
            Thought::WrongWork => "wrong work",
            Thought::Missing => "somebody gone",
            Thought::Plenty => "plenty",
            Thought::Cheered => "a good night",
        }
    }

    /// What holding this does to where a mood is headed. The two that can kill
    /// weigh most, and the one good thing here weighs no more than the lightest
    /// of the bad ones, which is the asymmetry the shape this is taken from has:
    /// what goes wrong is felt harder than what goes right.
    pub fn weight(self) -> f32 {
        match self {
            Thought::Cold => -18.0,
            Thought::Hungry => -20.0,
            Thought::Worn => -8.0,
            Thought::Bored => -7.0,
            Thought::WrongWork => -6.0,
            Thought::Missing => -10.0,
            Thought::Plenty => 6.0,
            Thought::Cheered => 7.0,
        }
    }
}

/// What a citizen is holding right now. A need counts only once it is past the
/// mark they act at -- the ordinary swing of a need is not something anybody
/// feels about, for the same reason it costs no focus.
pub fn thoughts_of(
    needs: &Needs,
    marks: &[Marks; NEED_COUNT],
    wrong_work: bool,
    somebody_missing: bool,
    plenty: bool,
    cheered: bool,
) -> [bool; THOUGHT_COUNT] {
    let past = |kind: NeedKind| needs.level(kind) < marks[kind as usize].low;
    let mut held = [false; THOUGHT_COUNT];
    held[Thought::Cold as usize] = past(NeedKind::Warmth);
    held[Thought::Hungry as usize] = past(NeedKind::Food);
    held[Thought::Worn as usize] = past(NeedKind::Rest);
    held[Thought::Bored as usize] = past(NeedKind::Recreation);
    held[Thought::WrongWork as usize] = wrong_work;
    held[Thought::Missing as usize] = somebody_missing;
    held[Thought::Plenty as usize] = plenty;
    held[Thought::Cheered as usize] = cheered;
    held
}

/// Where a mood is headed: the base everybody starts from, plus whatever is
/// being held. Instant, and never the number that is printed -- what is printed
/// is chasing this.
pub fn mood_target(held: &[bool; THOUGHT_COUNT], spared: bool) -> f32 {
    MOOD_BASE
        + THOUGHTS
            .into_iter()
            .filter(|thought| held[*thought as usize])
            .map(|thought| {
                let weight = thought.weight();
                // Only what hurts is discounted. What is going right is worth
                // the same to a survivor as to anybody else, because
                // expectations falling is about the floor and not the ceiling.
                if spared && weight < 0.0 {
                    weight * EXPECTATIONS_AFTER_LOSS
                } else {
                    weight
                }
            })
            .sum::<f32>()
}

/// Whether a season took enough of the colony to change what its survivors
/// measure against.
pub fn season_broke_them(began_with: usize, buried: u64) -> bool {
    began_with > 0 && buried as f32 >= began_with as f32 * LOSS_SHARE
}

/// When expectations come back up again.
pub fn spared_until(now: u64) -> u64 {
    now + EXPECTATIONS_SEASONS * ticks_per_season()
}

/// One tick of a mood chasing its target: quicker up than down, and standing
/// still while somebody is asleep, because a mood is about a day being lived
/// and a sleeping citizen is not living one.
///
/// The rates are per game day and not per game hour. At this tempo an hourly
/// rate puts every citizen on their target inside a morning, and a bar that is
/// always at its target is the target with extra arithmetic.
pub fn mood_step(mood: f32, target: f32, asleep: bool) -> f32 {
    if asleep {
        return mood;
    }
    let target = target.clamp(0.0, MOOD_MAX);
    let step = if target > mood {
        per_day(MOOD_RISE_PER_DAY)
    } else {
        -per_day(MOOD_FALL_PER_DAY)
    };
    if (target - mood).abs() <= step.abs() {
        target
    } else {
        (mood + step).clamp(0.0, MOOD_MAX)
    }
}

/// What a life has done to somebody, under the mood and much slower than it.
///
/// A mood is about a day and forgets one; this is what is left when enough days
/// have gone the same way, and it is measured in years because a thing that
/// heals in a season is a mood by another name. It is the clock the tradition
/// cards will run on when the culture layer arrives.
pub fn hardship_step(hardship: f32, mood: f32) -> f32 {
    let step = if mood < HARDSHIP_MARK {
        per_year(HARDSHIP_GAIN_PER_YEAR)
    } else if mood > HARDSHIP_EASE {
        -per_year(HARDSHIP_FADE_PER_YEAR)
    } else {
        0.0
    };
    (hardship + step).clamp(0.0, HARDSHIP_MAX)
}

/// What the colony would call that. Display only, like the focus bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hardship {
    Untouched,
    Worn,
    Marked,
    Broken,
}

impl Hardship {
    pub fn name(self) -> &'static str {
        match self {
            Hardship::Untouched => "untouched",
            Hardship::Worn => "worn",
            Hardship::Marked => "marked",
            Hardship::Broken => "broken",
        }
    }
}

pub fn hardship_status(hardship: f32) -> Hardship {
    if hardship < HARDSHIP_MAX * 0.1 {
        Hardship::Untouched
    } else if hardship < HARDSHIP_MAX * 0.4 {
        Hardship::Worn
    } else if hardship < HARDSHIP_MAX * 0.75 {
        Hardship::Marked
    } else {
        Hardship::Broken
    }
}

/// What a performance is worth, from the things that made it.
///
/// A sum of named contributions rather than a single figure, because the layer
/// this belongs to fails exactly when a role carries content instead of
/// consequence: a watcher has to be able to say why a night went well.
pub fn performance_quality(audience: usize, warmth: f32, performer: f32, tradition: f32) -> f32 {
    let heard = (audience as f32 / AUDIENCE_FULL).clamp(0.0, 1.0);
    let warm = (warmth / QUALITY_WARM_ENOUGH).clamp(0.0, 1.0);
    (heard * QUALITY_AUDIENCE
        + warm * QUALITY_WARMTH
        + performer.clamp(0.0, 1.0) * QUALITY_PERFORMER
        + tradition.clamp(0.0, 1.0) * QUALITY_TRADITION)
        .clamp(0.0, 1.0)
}

/// How a night went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Terrible,
    Boring,
    Fun,
    Unforgettable,
}

impl Outcome {
    pub fn name(self) -> &'static str {
        match self {
            Outcome::Terrible => "terrible",
            Outcome::Boring => "boring",
            Outcome::Fun => "fun",
            Outcome::Unforgettable => "unforgettable",
        }
    }

    /// How much of somebody's boredom a night like this lifts.
    pub fn worth(self) -> f32 {
        match self {
            Outcome::Terrible => 0.0,
            Outcome::Boring => 8.0,
            Outcome::Fun => 25.0,
            Outcome::Unforgettable => 40.0,
        }
    }

    /// How many days a night like this is still worth something for.
    pub fn mood_days(self) -> u64 {
        match self {
            Outcome::Fun => CHEER_DAYS_FUN,
            Outcome::Unforgettable => CHEER_DAYS_UNFORGETTABLE,
            _ => 0,
        }
    }

    pub fn went_well(self) -> bool {
        matches!(self, Outcome::Fun | Outcome::Unforgettable)
    }
}

/// Which of the four a performance of this quality turns out to be.
///
/// The shape is the ritual-outcome distribution this is taken from, kept as its
/// own arithmetic rather than approximated: a poor night can still go well and
/// a good one can still fall flat, which is what stops a colony treating an
/// entertainer as a switch.
pub fn performance_outcome(quality: f32, world: u64, salt: u64) -> Outcome {
    let quality = quality.clamp(0.0, 1.0);
    let roll = noise(world, PERFORMANCE_SALT.wrapping_add(salt));
    let terrible = 1.0 / (4.0 + 16.0 * quality);
    let boring = 3.0 / (4.0 + 16.0 * quality);
    let fun = 3.0 * quality / (1.0 + 4.0 * quality);
    if roll < terrible {
        Outcome::Terrible
    } else if roll < terrible + boring {
        Outcome::Boring
    } else if roll < terrible + boring + fun {
        Outcome::Fun
    } else {
        Outcome::Unforgettable
    }
}

/// The one multiplier every mismatch resolves into.
///
/// It is a ceiling of one that falls, and never a band that rises. Focus answers
/// what is wrong with a citizen; what is going right for a colony is paid
/// through a different channel entirely -- the colony-wide work-speed buff a
/// landed ritual hands out under ADR 0011 -- and the two have different readers.
/// Nothing thriving may be paid for in here, or there is no longer any state of
/// the colony that is invariant to compare a change against.
pub fn focus_of(needs: &Needs) -> f32 {
    let cost: f32 = NEEDS
        .into_iter()
        .filter(|kind| kind.is_survival())
        .map(|kind| focus_cost(kind, needs.level(kind)))
        .sum();
    let worst: f32 = NEEDS
        .into_iter()
        .filter(|kind| kind.is_survival())
        .map(|kind| kind.rules().weight * (1.0 + FOCUS_KNEE_BITE))
        .sum();
    1.0 - (1.0 - FOCUS_FLOOR) * (cost / worst)
}

/// What the colony would call that number. Four bands and not the five the
/// shape is borrowed from, because the fifth is the bonus half and there is no
/// bonus half here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Focused,
    Unfocused,
    Distracted,
    BadlyDistracted,
}

impl Focus {
    pub fn name(self) -> &'static str {
        match self {
            Focus::Focused => "focused",
            Focus::Unfocused => "unfocused",
            Focus::Distracted => "distracted",
            Focus::BadlyDistracted => "badly distracted",
        }
    }
}

pub fn focus_band(focus: f32) -> Focus {
    if focus >= FOCUS_UNFOCUSED {
        Focus::Focused
    } else if focus >= FOCUS_DISTRACTED {
        Focus::Unfocused
    } else if focus >= FOCUS_BADLY {
        Focus::Distracted
    } else {
        Focus::BadlyDistracted
    }
}

/// What a citizen brings to the work: what the colony raised in them, scaled by
/// how much of themselves they have to give today and by how well they fit the
/// trade. The one product every consequence of a mismatch comes out of.
pub fn effective_stat(base: f32, focus_mult: f32, trade_fit_mult: f32) -> f32 {
    base * focus_mult * trade_fit_mult
}

/// What one trip brings home, and what is left banked toward the next. A slope
/// rather than a step: the fraction is carried forward rather than rolled for,
/// so a small difference in what the colony raised is a small difference in
/// what comes back, and the run still replays exactly. This is the only place
/// anything the colony raised turns into capacity rather than survival.
pub fn haul_load(effective: f32, banked: f32) -> (u32, f32) {
    let earned = banked + HAUL_BASE + effective.max(0.0) * HAUL_LOAD_SWING;
    let whole = earned.floor();
    (whole as u32, earned - whole)
}

/// How good a candidate a citizen is for a vacancy.
pub fn assignment_score(distance: i32, experience: f32, bias: f32) -> f32 {
    WEIGHT_EXPERIENCE * experience + WEIGHT_BIAS * bias - WEIGHT_DISTANCE * distance as f32
}

/// The vacancy goes to one of the best few rather than the best.
pub fn pick_from_top(scored: &[(usize, f32)], roll: f32) -> Option<usize> {
    if scored.is_empty() {
        return None;
    }
    let mut ranked = scored.to_vec();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let shortlist = ranked.len().min(ASSIGNMENT_SHORTLIST);
    let pick = ((roll * shortlist as f32) as usize).min(shortlist - 1);
    Some(ranked[pick].0)
}

/// One hour of a trade being practised, or of another one going to rust.
pub fn experience_step(experience: f32, working: bool) -> f32 {
    let moved = if working {
        experience + EXPERIENCE_GAIN
    } else {
        experience - EXPERIENCE_RUST
    };
    moved.clamp(0.0, 1.0)
}

/// How many hands the colony keeps out of the trades so somebody can always
/// take work no trade covers.
/// How many of the colony's hands it spares for amusing the rest. One in a
/// founding party, and a share after that -- an hour spent entertaining is an
/// hour not spent on wood, so this is deliberately the smallest trade.
pub fn entertainers_wanted(hands: usize) -> usize {
    ((hands as f32 * ENTERTAINER_SHARE).round() as usize).max(1)
}

pub fn laborers_wanted(hands: usize) -> usize {
    (hands as f32 * LABORER_SHARE).round() as usize
}

/// What a child of the house starts with. The household is the vertical channel
/// and this is the whole of what it hands down.
pub fn inherited_experience(head: f32) -> f32 {
    head * INHERITED_SHARE
}

/// The oldest grown citizen of a house, whose trade a child of it takes. The
/// household is the vertical channel and this is the whole of what runs it.
pub fn household_head(adults: &[(IVec2, Trade, f32, f32)], home: IVec2) -> Option<(Trade, f32)> {
    adults
        .iter()
        .filter(|(house, ..)| *house == home)
        .max_by(|a, b| a.3.total_cmp(&b.3))
        .map(|(_, trade, experience, _)| (*trade, *experience))
}

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
    /// The whole childhood rather than the current stage, which is what decides
    /// how much of the citizen the colony can claim credit for.
    lived_warmth: f32,
    lived_food: f32,
    lived_hours: f32,
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
            lived_warmth: 0.0,
            lived_food: 0.0,
            lived_hours: 0.0,
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
        self.lived_warmth += warmth;
        self.lived_food += food;
        self.lived_hours += 1.0;
    }

    /// How good the whole childhood was. A childhood nobody watched reads as
    /// neither good nor bad, which is the only honest answer for a migrant.
    pub fn prosperity(&self) -> f32 {
        if self.lived_hours == 0.0 {
            return FORMATION_NEUTRAL;
        }
        (self.lived_warmth + self.lived_food) / (2.0 * self.lived_hours)
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
    /// Returns whether this is the settling that finished the childhood, which
    /// is when a citizen's span stops being a guess.
    pub fn settle_due(&mut self, age: f32) -> bool {
        if self.settled >= MILESTONE_COUNT || age < MILESTONE_AGES[self.settled] {
            return false;
        }
        let hours = self.hours.max(1.0);
        self.resolve(age, self.warmth / hours, self.food / hours);
        self.warmth = 0.0;
        self.food = 0.0;
        self.hours = 0.0;
        self.settled >= MILESTONE_COUNT
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
    Recreation,
}

pub const NEEDS: [NeedKind; NEED_COUNT] = [
    NeedKind::Warmth,
    NeedKind::Rest,
    NeedKind::Food,
    NeedKind::Recreation,
];

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
    /// What this need is worth against the others when focus is worked out.
    pub weight: f32,
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
                weight: NEED_WEIGHT_SURVIVAL,
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
                weight: NEED_WEIGHT_SURVIVAL,
                fatal: false,
            },
            // Slow, survivable, and answered by somebody's time rather than by
            // anything the colony can build. It falls over weeks and fills in an
            // evening, which is what makes an entertainer's hour worth spending
            // and a missed one worth nothing much.
            NeedKind::Recreation => NeedRules {
                decay: per_day(4.0),
                // Nothing about standing anywhere fills this one: it is filled
                // by attending a performance, which is an event and not a
                // state, so the ordinary recovery term is nought and `amuse`
                // does the work.
                recovery: 0.0,
                low: 35.0,
                high: 85.0,
                comfort: 50.0,
                weight: NEED_WEIGHT_COMFORT,
                fatal: false,
            },
            NeedKind::Food => NeedRules {
                decay: per_day(7.0),
                recovery: per_hour(60.0),
                low: 30.0,
                high: 90.0,
                comfort: 40.0,
                weight: NEED_WEIGHT_SURVIVAL,
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

/// Where a need's two marks stand for one citizen right now: the level they
/// start acting at and the level they stop at.
#[derive(Debug, Clone, Copy)]
pub struct Marks {
    pub low: f32,
    pub high: f32,
}

/// The marks a citizen is working to, given what the walk home would cost them
/// and the margin they keep on top of it.
///
/// Warmth is the one need answered by walking, so it is the one whose marks have
/// to know where the citizen is standing: a controller comparing a level against
/// a fixed number has no term for how far away its actuator is, and turns for
/// home on schedule from a distance that schedule cannot cover. ADR 0003 is
/// amended for this -- a threshold is a constant or a pure function of position,
/// and what the ADR guarantees is the gap between the marks, not their value.
pub fn marks_of(kind: NeedKind, home_cost: f32, margin: f32) -> Marks {
    let rules = kind.rules();
    match kind {
        NeedKind::Warmth => Marks {
            low: home_cost + margin,
            high: home_cost + margin + (rules.high - rules.low),
        },
        _ => Marks {
            low: rules.low,
            high: rules.high,
        },
    }
}

/// Every need's marks for one citizen, worked out once so that nothing asks the
/// same question twice in a tick.
pub fn marks_for(home_cost: f32, margin: f32) -> [Marks; NEED_COUNT] {
    let mut marks = [Marks {
        low: 0.0,
        high: 0.0,
    }; NEED_COUNT];
    for kind in NEEDS {
        marks[kind as usize] = marks_of(kind, home_cost, margin);
    }
    marks
}

/// How far through its own tolerance band a need at `level` has fallen: zero at
/// the mark where a citizen stops acting on it, one where they start.
pub fn shortfall_at(marks: Marks, level: f32) -> f32 {
    (marks.high - level) / (marks.high - marks.low)
}

pub fn need_step(need: Need, kind: NeedKind, met: bool, decay_scale: f32, marks: Marks) -> Need {
    let rules = kind.rules();
    let level = if met {
        (need.level + rules.recovery).min(NEED_MAX)
    } else {
        (need.level - rules.decay * decay_scale).max(0.0)
    };
    let pressing = if level <= marks.low {
        true
    } else if level >= marks.high {
        false
    } else {
        need.pressing
    };
    Need {
        level,
        pressing,
        // Only the part past the point of acting counts: a citizen already on
        // their way to the fire or the granary is not being failed yet.
        burden: need.burden + (shortfall_at(marks, level) - 1.0).max(0.0),
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
    /// Where those walks were walked, summed as offsets from the hearth. The
    /// count alone says a citizen is losing hours; this says whether moving a
    /// fire would give them back, and where to move it to.
    detour_from: IVec2,
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
        let mut needs = Needs {
            needs,
            detour: 0,
            detour_from: IVec2::ZERO,
        };
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

    pub fn step(&mut self, kind: NeedKind, met: bool, decay_scale: f32, marks: Marks) {
        self.needs[kind as usize] = need_step(self.get(kind), kind, met, decay_scale, marks);
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
    pub fn shortfall(&self, kind: NeedKind, marks: Marks) -> f32 {
        shortfall_at(marks, self.level(kind))
    }

    /// Draw a line under the season: from here the ballot remembers only what
    /// happens next, and the hours it counts start again from none.
    pub fn forget_before_ballot(&mut self) {
        for kind in NEEDS {
            self.needs[kind as usize].burden = 0.0;
        }
        self.detour = 0;
        self.detour_from = IVec2::ZERO;
    }

    /// One tick of a citizen's ballot window, and whether it went on getting
    /// warm rather than on work.
    pub fn spend(&mut self, on_getting_warm: bool, at: IVec2) {
        if on_getting_warm {
            self.detour = self.detour.saturating_add(1);
            self.detour_from += at - CENTER;
        }
    }

    /// The middle of the walks back to the fire this citizen has made, if they
    /// have made any. Walks scattered evenly around the hearth average out to
    /// the hearth itself, which is the honest answer: there is no one place to
    /// put a fire for somebody who loses hours in every direction.
    pub fn detour_middle(&self) -> Option<IVec2> {
        (self.detour > 0).then(|| CENTER + self.detour_from / self.detour as i32)
    }

    /// What the walk back to the fire has cost, in the same currency as a need:
    /// one tick of it weighs what one tick at the point of acting on a need
    /// weighs, so every entry on the ballot compares directly.
    pub fn detour_burden(&self) -> f32 {
        self.detour as f32
    }

    /// What a performance lifts. Recreation is the one need filled by something
    /// happening rather than by somebody standing somewhere, so it is raised
    /// here rather than through the ordinary recovery term.
    pub fn amuse(&mut self, worth: f32) {
        let need = &mut self.needs[NeedKind::Recreation as usize];
        need.level = (need.level + worth).min(NEED_MAX);
    }

    /// Whether nothing the colony runs on has gone unanswered since the last
    /// ballot. It is the test the survival tier of the vote applies, asked
    /// directly: a citizen it has nothing to say about is one the colony can
    /// spare for looking.
    ///
    /// A comfort need is not on this list. One that nothing a citizen does
    /// alone can raise would never clear, and the colony would quietly stop
    /// sending anybody out to look -- which is a whole layer switched off by a
    /// need that was meant to cost a mood.
    pub fn nothing_failed(&self) -> bool {
        NEEDS
            .into_iter()
            .filter(|kind| kind.is_survival())
            .all(|kind| self.get(kind).burden == 0.0)
    }

    pub fn comfortable(&self, kind: NeedKind) -> bool {
        self.level(kind) >= kind.rules().comfort
    }

    /// Pressing needs, worst first. The sort is stable, so equally short needs
    /// keep `NEEDS` order and the same colony state always decides the same way.
    pub fn pressing_by_urgency(&self, marks: &[Marks; NEED_COUNT]) -> Vec<NeedKind> {
        let mut pressing: Vec<NeedKind> = NEEDS
            .into_iter()
            .filter(|kind| self.get(*kind).pressing)
            .collect();
        // Weight first and shortfall second, so a need nobody dies of can never
        // take a citizen away from one that kills however far it has fallen.
        // The ordering reads that off the weight rather than off a rule naming
        // which needs are which.
        pressing.sort_by(|a, b| {
            b.rules().weight.total_cmp(&a.rules().weight).then(
                self.shortfall(*b, marks[*b as usize])
                    .total_cmp(&self.shortfall(*a, marks[*a as usize])),
            )
        });
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
/// Every waystation shed the colony keeps. Held apart from the citizens by
/// `Without`, because both want a position and only one may have it mutably.
pub type Sheds<'w, 's> =
    Query<'w, 's, (&'static Pos, &'static mut Cache), (With<Structure>, Without<Citizen>)>;

#[derive(SystemParam)]
pub struct Colony<'w, 's> {
    pub generator: ResMut<'w, Generator>,
    pub granary: ResMut<'w, Granary>,
    pub patches: ResMut<'w, Patches>,
    pub construction: ResMut<'w, Construction>,
    /// The sheds at the waystations, which are a store like any other and are
    /// filled by the same haulers.
    pub posts: Sheds<'w, 's>,
    /// Who has not come back.
    pub missing: ResMut<'w, Missing>,
    /// Who has just stopped being here, which the weather asks about.
    pub toll: ResMut<'w, Toll>,
    /// Where tonight's performances will be, so a citizen can decide to go.
    pub stages: Res<'w, Stages>,
}

/// Everything about the air, gathered into one borrow: what the weather is
/// doing, what it has cost, and what the colony is standing in because of it.
#[derive(SystemParam)]
pub struct Sky<'w> {
    pub weather: ResMut<'w, Weather>,
    pub toll: ResMut<'w, Toll>,
    pub air: ResMut<'w, Air>,
}

/// Everything the colony has put by, gathered into one borrow so readers do not
/// have to name each store separately.
#[derive(SystemParam)]
pub struct Stores<'w> {
    pub generator: Res<'w, Generator>,
    pub granary: Res<'w, Granary>,
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

/// The world as the colony has met it.
///
/// A chunk is realised the first time somebody looks inside it and may be
/// dropped again when nobody is near, because the seed can always redraw it.
/// What the colony took is the one thing the seed cannot know, so it is kept
/// apart and applied on top -- which is what stops a colony stripping a
/// treeline, walking away, and coming back to a full one.
///
/// Chunks are held in sorted order so that a walk over the map itself runs in a
/// fixed one. The queries below do not lean on it -- they step an explicit range
/// of coordinates and look each chunk up by key -- but nothing should have to
/// work out which of the two it is looking at to know that a run replays.
/// One chunk as the colony holds it: what stands in it, and how much of each
/// kind, so that a search can pass over a chunk without looking inside it.
struct Held {
    patches: Vec<Patch>,
    standing: [u32; CARGO_COUNT],
}

impl Held {
    fn new(patches: Vec<Patch>) -> Self {
        let mut standing = [0u32; CARGO_COUNT];
        for patch in &patches {
            standing[patch.kind as usize] += patch.amount;
        }
        Self { patches, standing }
    }
}

#[derive(Resource)]
pub struct Patches {
    seed: u64,
    chunks: BTreeMap<(i32, i32), Held>,
    worked: BTreeMap<(i32, i32), u32>,
    /// Chunks somebody has stood in. Knowing a chunk is not the same as holding
    /// it: memory is dropped when nobody is near and drawn again from the seed,
    /// but the colony does not forget that it has been somewhere. Haulers work
    /// known ground only, which is the whole of what a scout's walk buys.
    known: BTreeSet<(i32, i32)>,
}

/// The Chebyshev distance from a cell to the nearest cell of a chunk: how close
/// anything in that chunk could possibly be. Nothing inside is opened to know
/// it, which is what lets a search skip a chunk unrealised.
fn box_distance(from: IVec2, chunk: IVec2) -> i32 {
    let low = chunk * CHUNK;
    let high = low + IVec2::splat(CHUNK - 1);
    let across = (low.x - from.x).max(from.x - high.x).max(0);
    let down = (low.y - from.y).max(from.y - high.y).max(0);
    across.max(down)
}

/// The chunks at one Chebyshev step out from a middle one.
fn chunk_ring(middle: IVec2, ring: i32) -> Vec<IVec2> {
    if ring == 0 {
        return vec![middle];
    }
    let mut out = Vec::with_capacity((8 * ring) as usize);
    for x in -ring..=ring {
        for y in -ring..=ring {
            if x.abs() == ring || y.abs() == ring {
                out.push(middle + IVec2::new(x, y));
            }
        }
    }
    out
}

fn key(at: IVec2) -> (i32, i32) {
    (at.x, at.y)
}

impl Patches {
    pub fn new(seed: u64) -> Self {
        let mut world = Self {
            seed,
            chunks: BTreeMap::new(),
            worked: BTreeMap::new(),
            known: BTreeSet::new(),
        };
        // A founding party knows the ground it lives off and nothing beyond it.
        // Everything further has to be walked to before it can be worked.
        let low = chunk_of(CENTER - IVec2::splat(NEAR_GROUND));
        let high = chunk_of(CENTER + IVec2::splat(NEAR_GROUND));
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                world.discover(IVec2::new(x, y) * CHUNK);
            }
        }
        world
    }

    /// Whether anybody has stood in this chunk.
    pub fn has_been_to(&self, chunk: IVec2) -> bool {
        self.known.contains(&key(chunk))
    }

    /// Somebody is standing here, so the colony knows this ground from now on.
    pub fn discover(&mut self, at: IVec2) {
        let chunk = chunk_of(at);
        self.known.insert(key(chunk));
        self.realise(chunk);
    }

    /// Everything the colony is holding, counted in cells so that the two
    /// halves can be added at all. A realised chunk is the whole square; a
    /// remembered cut is the one cell it was made in.
    #[cfg(not(feature = "window"))]
    pub fn held_cells(&self) -> usize {
        self.chunks.len() * (CHUNK * CHUNK) as usize + self.worked.len() + self.known.len()
    }

    fn realise(&mut self, chunk: IVec2) {
        if self.chunks.contains_key(&key(chunk)) {
            return;
        }
        let mut patches = chunk_patches(self.seed, chunk);
        for patch in &mut patches {
            if let Some(left) = self.worked.get(&key(patch.pos)) {
                patch.amount = *left;
            }
        }
        self.chunks.insert(key(chunk), Held::new(patches));
    }

    /// What stands within a reach of here that the colony has already met.
    /// Reading the world never makes more of it.
    pub fn seen(&self, at: IVec2, radius: i32) -> impl Iterator<Item = &Patch> {
        let low = chunk_of(at - IVec2::splat(radius));
        let high = chunk_of(at + IVec2::splat(radius));
        (low.x..=high.x)
            .flat_map(move |x| (low.y..=high.y).map(move |y| (x, y)))
            .filter_map(move |chunk| self.chunks.get(&chunk))
            .flat_map(|held| held.patches.iter())
            .filter(move |patch| (patch.pos - at).abs().max_element() <= radius)
    }

    fn patch_at(&mut self, pos: IVec2) -> Option<&mut Patch> {
        self.realise(chunk_of(pos));
        self.chunks
            .get_mut(&key(chunk_of(pos)))?
            .patches
            .iter_mut()
            .find(|patch| patch.pos == pos)
    }

    /// Strips a patch of what a hauler actually lifted, and hands back what was
    /// there to lift. What leaves the ground has to equal what reaches the store.
    pub fn take(&mut self, pos: IVec2, wanted: u32) -> u32 {
        let Some(patch) = self.patch_at(pos) else {
            return 0;
        };
        let taken = wanted.min(patch.amount);
        patch.amount -= taken;
        let (left, kind) = (patch.amount, patch.kind);
        if taken > 0 {
            self.worked.insert(key(pos), left);
            if let Some(held) = self.chunks.get_mut(&key(chunk_of(pos))) {
                held.standing[kind as usize] -= taken;
            }
        }
        taken
    }

    /// What one chunk has standing of a kind, without opening it.
    pub fn standing_in(&mut self, chunk: IVec2, kind: Cargo) -> u32 {
        self.realise(chunk);
        self.chunks
            .get(&key(chunk))
            .map_or(0, |held| held.standing[kind as usize])
    }

    /// The nearest patch of a kind, found by opening chunks in the order they
    /// could possibly hold something close rather than by scanning a radius.
    ///
    /// The cost of a query is set by how far the answer turns out to be, not by
    /// how far a citizen is willing to look: rings of chunks are taken in turn,
    /// a ring is abandoned the moment nothing in it could beat what is already
    /// found, and a chunk that says it holds none of that kind is passed over
    /// without being looked inside.
    pub fn nearest(&mut self, kind: Cargo, from: IVec2, limit: i32) -> Option<IVec2> {
        let home = chunk_of(from);
        let mut best: Option<(i32, IVec2)> = None;
        for ring in 0.. {
            let chunks = chunk_ring(home, ring);
            let floor = chunks
                .iter()
                .map(|chunk| box_distance(from, *chunk))
                .min()
                .unwrap_or(i32::MAX);
            if floor > limit || best.is_some_and(|(walk, _)| floor > walk) {
                break;
            }
            for chunk in chunks {
                // Skipped chunk by chunk and not ring by ring: a chunk whose
                // nearest cell is further than what is already found cannot
                // improve on it, and most of a ring is in that position.
                let reach = box_distance(from, chunk);
                if reach > limit || best.is_some_and(|(walk, _)| reach >= walk) {
                    continue;
                }
                if !self.has_been_to(chunk) || self.standing_in(chunk, kind) == 0 {
                    continue;
                }
                let Some(held) = self.chunks.get(&key(chunk)) else {
                    continue;
                };
                for patch in &held.patches {
                    if patch.kind != kind || patch.amount == 0 {
                        continue;
                    }
                    let walk = (patch.pos - from).abs().max_element();
                    if walk <= limit && best.is_none_or(|(found, _)| walk < found) {
                        best = Some((walk, patch.pos));
                    }
                }
            }
        }
        best.map(|(_, pos)| pos)
    }

    /// A day's growing back, over the world the colony is holding. Ground it has
    /// never been to does not grow, because it was never taken from.
    pub fn regrow(&mut self, growing: bool) {
        if !growing {
            return;
        }
        for held in self.chunks.values_mut() {
            let mut standing = [0u32; CARGO_COUNT];
            for patch in held.patches.iter_mut() {
                patch.amount = regrowth_step(patch.amount, patch.cap, patch.kind, true);
                standing[patch.kind as usize] += patch.amount;
                // A patch back at its cap is what the seed already says it is,
                // so remembering it is remembering nothing.
                if patch.amount == patch.cap {
                    self.worked.remove(&key(patch.pos));
                } else if let Some(left) = self.worked.get_mut(&key(patch.pos)) {
                    *left = patch.amount;
                }
            }
            held.standing = standing;
        }
    }

    /// Drops the world nobody is standing near. What was taken survives, so a
    /// chunk asked for again comes back as it was left rather than as new.
    pub fn forget_beyond(&mut self, homes: &[IVec2], radius: i32) {
        let span = radius / CHUNK + 1;
        self.chunks.retain(|(x, y), _| {
            homes.iter().any(|home| {
                let near = chunk_of(*home);
                (near.x - x).abs() <= span && (near.y - y).abs() <= span
            })
        });
    }
}

/// Somebody who went out and did not come back.
///
/// The colony does not learn that a citizen died. It learns that they are not
/// here, and where they were last making for; what closes the event is either a
/// body found or a slab raised without one, and the second costs build capacity
/// where the first costs somebody the same walk.
#[derive(Debug, Clone, Copy)]
pub struct Lost {
    pub at: IVec2,
    pub since: u64,
}

#[derive(Resource, Default)]
pub struct Missing(Vec<Lost>);

impl Missing {
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Somebody has stopped being here. Whether that is a death or an absence
    /// depends only on whether the colony was in a position to see it.
    pub fn take_note(&mut self, at: IVec2, now: u64) {
        if goes_missing(at) {
            self.lost(at, now);
        }
    }

    /// Kept private on purpose: production records an absence through
    /// `take_note`, which is where the rule about being seen lives.
    fn lost(&mut self, at: IVec2, now: u64) {
        self.0.push(Lost { at, since: now });
    }

    /// The spot nearest to somebody setting out, if anybody is unaccounted for.
    pub fn nearest_to(&self, from: IVec2) -> Option<IVec2> {
        self.0
            .iter()
            .min_by_key(|lost| ((lost.at - from).abs().max_element(), lost.at.x, lost.at.y))
            .map(|lost| lost.at)
    }

    /// Anybody standing where somebody was lost has found them.
    pub fn recover(&mut self, standing: &[IVec2]) {
        self.0.retain(|lost| {
            !standing
                .iter()
                .any(|at| (*at - lost.at).abs().max_element() <= 1)
        });
    }

    /// What the colony gives up on, and pays to give up on. A slab closes the
    /// event without a body; a colony that cannot spare the wood keeps waiting,
    /// which is the honest version of not being able to afford to grieve.
    pub fn raise_memorials(&mut self, now: u64, wood: &mut u32) {
        self.0.retain(|lost| {
            if now.saturating_sub(lost.since) < MEMORIAL_AFTER || *wood < MEMORIAL_WOOD_COST {
                return true;
            }
            *wood -= MEMORIAL_WOOD_COST;
            false
        });
    }
}

/// Whether somebody dying here is a death the colony sees or an absence it is
/// left to work out. Past the ground it works, nobody is watching.
pub fn goes_missing(at: IVec2) -> bool {
    (at - CENTER).abs().max_element() > NEAR_GROUND
}

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
    Waystation,
}

pub const BUILDINGS: [Building; BUILDING_COUNT] = [
    Building::House,
    Building::HuntersHut,
    Building::GeneratorUpgrade,
    Building::Waystation,
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
            Building::Waystation => BuildingRules {
                cost: WAYSTATION_WOOD_COST,
                name: "Post",
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
    /// The same thumb, on the other scale: which trades the office leans toward
    /// when a vacancy is filled. Inert data, like the rest of the office.
    pub trade_bias: [f32; TRADE_COUNT],
}

/// The last ballot the colony held, kept so it can be shown.
#[derive(Resource, Default)]
pub struct Ballot {
    pub tally: [f32; BUILDING_COUNT],
}

/// A finished building standing on a plot.
#[derive(Component)]
pub struct Structure(pub Building);

/// The woodshed at a waystation. The brazier burns from it and haulers fill it,
/// which is what makes a chain of them a standing tax on the workforce rather
/// than a thing built once and forgotten.
#[derive(Component, Default)]
pub struct Cache(pub u32);

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
    /// How far their body has learned the cold, which the cold takes back.
    pub acclimated: f32,
    /// Days of work the colony has actually watched them do, which is what its
    /// opinion of them is worth.
    pub watched: f32,
    /// The part of a load they were owed and could not carry, kept for the next
    /// trip so nothing is lost to rounding.
    pub banked: f32,
    /// How much they lifted, decided when they lifted it.
    pub load: u32,
    /// What they do with their days, and what they have practised at.
    pub trade: Trade,
    pub experience: [f32; TRADE_COUNT],
    /// How this one is bearing up. Nothing in the simulation reads it yet: the
    /// ballot keeps its own two currencies -- what a need cost and what a walk
    /// wasted -- until the culture layer gives a mood somewhere to go. It is
    /// printed so that what is happening to people is visible before anything
    /// acts on it.
    pub mood: f32,
    /// What they are holding that makes it. Kept beside the mood rather than
    /// worked out again by whoever prints it, because the inputs are half the
    /// colony and a second copy of that sum is a second answer waiting to differ.
    pub held: [bool; THOUGHT_COUNT],
    /// What the years have done to them, under the mood and far slower. The
    /// culture layer will read this; nothing does yet.
    pub hardship: f32,
    /// Until when the last good night is still worth something to them.
    pub cheered_until: u64,
    /// Until when this one is measuring against a worse year than the colony
    /// used to have. Set by a winter that took a share of the colony, and it
    /// runs out.
    pub spared_until: u64,
    /// Where this citizen is walking to look, if the colony has sent them out.
    /// A scout brings back the map and nothing else.
    pub scouting: Option<IVec2>,
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
    /// Walking to tonight's performance. The only duty that answers a need
    /// nobody can meet alone, and the only one that is not work.
    Attend,
}

/// Deterministic noise in `[0, 1)` from a pair of integers. The simulation has
/// no entropy source and wants none: two runs of the same build must tell the
/// same story, or a balance log is worth nothing.
pub fn mix(seed: u64, salt: u64) -> u64 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub fn noise(seed: u64, salt: u64) -> f32 {
    (mix(seed, salt) >> 40) as f32 / (1u64 << 24) as f32
}

/// How much of a citizen the colony cannot account for. A starving colony can
/// read the differences between its people straight off the granary; a
/// comfortable one raises people who differ for reasons it never saw.
pub fn residual_share(prosperity: f32) -> f32 {
    RESIDUAL_SHARE_LEAN + prosperity.clamp(0.0, 1.0) * (RESIDUAL_SHARE_FAT - RESIDUAL_SHARE_LEAN)
}

/// The span a citizen would reach if nothing else got them first: what the
/// colony raised in them, and a part it cannot account for, weighed against
/// each other by how comfortable the colony was while raising them.
pub fn lifespan_of(seed: u64, raised: f32, prosperity: f32) -> f32 {
    let earned = LIFESPAN_BASE
        * (1.0 + (raised - FORMATION_NEUTRAL) / FORMATION_NEUTRAL * LIFESPAN_RAISED_SWING);
    let unexplained =
        LIFESPAN_BASE * (1.0 + (noise(seed, LIFESPAN_SALT) - 0.5) * 2.0 * LIFESPAN_SPREAD);
    let share = residual_share(prosperity);
    earned * (1.0 - share) + unexplained * share
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
/// One hour of a body either learning the cold or forgetting it.
pub fn acclimation_step(acclimated: f32, in_the_cold: bool) -> f32 {
    let moved = if in_the_cold {
        acclimated + ACCLIMATION_GAIN
    } else {
        acclimated - ACCLIMATION_LOSS
    };
    moved.clamp(0.0, 1.0)
}

/// What actually stands between a citizen and a cold night: what age has left
/// them, scaled by what the colony raised in them, plus what the winters have
/// taught their body since. Only the last term survives their own span, because
/// it is not something age took away.
pub fn cold_resistance(age: f32, lifespan: f32, raised: f32, acclimated: f32) -> f32 {
    let left = hardiness(age, lifespan);
    let bodily = raised / FORMATION_NEUTRAL;
    (left * bodily + acclimated * ACCLIMATION_WORTH).clamp(0.0, 1.0)
}

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

/// The climate on the day a tick falls in: a cosine through the year, so the
/// drift from one day to the next is smooth and the extremes land mid-season
/// rather than on a boundary. It is what a day is on average, and no hour of
/// that day is actually at it.
pub fn climate_at(tick: u64) -> f32 {
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

/// What the day does on top of the year: coldest in the small hours, warmest
/// twelve hours later, and averaging over a whole day to exactly the climate it
/// sits on, so the seasons are untouched by the days having a shape.
///
/// It does not scale with the severity ramp the winters do. The ramp is about
/// how hard a year is; this is about the difference between noon and the small
/// hours, which is the same difference in a colony's first year as in its
/// fortieth.
pub fn diurnal_at(tick: u64) -> f32 {
    let hour = tick / TICKS_PER_HOUR % HOURS_PER_DAY;
    let warmest = (COLDEST_HOUR + HOURS_PER_DAY / 2) % HOURS_PER_DAY;
    let phase = (hour as f32 - warmest as f32) / HOURS_PER_DAY as f32 * std::f32::consts::TAU;
    DIURNAL_SWING / 2.0 * phase.cos()
}

/// One day's step of the weather away from the climate. Deterministic in the
/// world seed and the day, so a run replays: the same world gets the same
/// weather every time it is founded.
pub fn front_step(front: f32, world: u64, day: u64) -> f32 {
    // What survives a day, derived from how long a front is said to last rather
    // than stated a second time beside it: the two would drift apart and only
    // one of them would be the one the weather actually used.
    let memory = (-1.0 / FRONT_DAYS).exp();
    let push = (noise(world, FRONT_SALT.wrapping_add(day)) * 2.0 - 1.0) * FRONT_STEP;
    (front * memory + push).clamp(-FRONT_CAP, FRONT_CAP)
}

/// What the weather is doing beyond wandering: a spell of it, with a beginning
/// and an end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spell {
    ColdSnap,
    Blizzard,
    Thaw,
}

/// One spell, as the colony is living through it.
#[derive(Debug, Clone, Copy)]
pub struct Weathering {
    pub kind: Spell,
    /// How far it takes the air at its worst, always a positive number: which
    /// way it goes is the kind's business.
    pub depth: f32,
    pub began: u64,
    pub days: u64,
}

impl Weathering {
    /// What this spell is doing to the air on a given day, signed. It comes on
    /// over the onset, holds, and goes off again over the same, so a watcher
    /// sees it approaching and sees it leave rather than meeting a step twice.
    pub fn air_on(&self, day: u64) -> f32 {
        if day <= self.began || day >= self.began + self.days {
            return 0.0;
        }
        let through = day - self.began;
        let left = self.days - through;
        let ramp = (through.min(SPELL_ONSET_DAYS) as f32 / SPELL_ONSET_DAYS as f32)
            .min(left.min(SPELL_ONSET_DAYS) as f32 / SPELL_ONSET_DAYS as f32);
        let depth = self.depth * ramp;
        match self.kind {
            Spell::Thaw => depth,
            _ => -depth,
        }
    }
}

/// What the colony would call the sky today.
pub fn weather_word(spell: Option<&Weathering>, day: u64) -> &'static str {
    match spell {
        Some(spell) if spell.air_on(day) != 0.0 => match spell.kind {
            Spell::ColdSnap => "snow",
            Spell::Blizzard => "blizzard",
            Spell::Thaw => "thaw",
        },
        _ => "clear",
    }
}

/// What the colony can be hit with, in the degrees the air is measured in.
///
/// Budgeted rather than rolled, and budgeted from the colony's own success:
/// mouths to feed and stores per head, held down through the first year by the
/// ramp the winters already use, and eased by the grace a colony earns for not
/// burying anybody. Capped, always, because the rule of this layer is to hurt
/// and not to kill.
pub fn severity_budget(population: usize, fuel: u32, food: u32, year: u64, adaptation: f32) -> f32 {
    let share = stock_share(fuel, FUEL_PER_CITIZEN, population)
        .min(stock_share(food, FOOD_PER_CITIZEN, population))
        .clamp(0.0, BUDGET_SHARE_CAP);
    let standing = BUDGET_FLOOR + population as f32 * BUDGET_PER_HEAD + share * BUDGET_PER_SHARE;
    (standing * severity(year) * adaptation).clamp(0.0, BUDGET_CEILING)
}

/// One tick of the grace a colony earns by not burying anybody.
pub fn adaptation_step(adaptation: f32, a_death: bool) -> f32 {
    let moved = if a_death {
        adaptation - ADAPT_DEATH_COST
    } else {
        adaptation + per_year(ADAPT_RISE_PER_YEAR)
    };
    moved.clamp(ADAPT_MIN, ADAPT_MAX)
}

/// Whether the weather turns today, and into what. Drawn from the world's own
/// seed and the day, so a run replays; how often and how deep both come off the
/// budget, so a colony with little sees weather rarely and shallowly.
pub fn spell_due(budget: f32, world: u64, day: u64) -> Option<Weathering> {
    let room = (budget / BUDGET_CEILING).clamp(0.0, 1.0);
    if noise(world, SPELL_SALT.wrapping_add(day)) >= SPELL_CHANCE_PER_DAY * room {
        return None;
    }
    let draw = noise(world, SPELL_SALT.wrapping_add(day).wrapping_mul(3));
    let depth = budget * (0.4 + 0.6 * draw);
    let length = noise(world, SPELL_SALT.wrapping_add(day).wrapping_mul(5));
    let days = SPELL_DAYS_MIN + (length * (SPELL_DAYS_MAX - SPELL_DAYS_MIN) as f32) as u64;
    let kind = if noise(world, SPELL_SALT.wrapping_add(day).wrapping_mul(7)) < THAW_SHARE {
        Spell::Thaw
    } else if depth >= BUDGET_CEILING * BLIZZARD_SHARE {
        Spell::Blizzard
    } else {
        Spell::ColdSnap
    };
    Some(Weathering {
        kind,
        depth,
        began: day,
        days,
    })
}

/// How far the weather has wandered from the climate, and when it last moved.
/// The front needs a day of memory, which is why it lives here rather than
/// being a function of the tick like the season and the hour are.
#[derive(Resource)]
pub struct Weather {
    pub front: f32,
    pub day: u64,
    /// The grace the colony has earned by not burying anybody.
    pub adaptation: f32,
    /// The spell it is living through, if any.
    pub spell: Option<Weathering>,
    /// What the sky is doing, as the colony would say it.
    pub word: &'static str,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            front: 0.0,
            day: 0,
            adaptation: 1.0,
            spell: None,
            word: "clear",
        }
    }
}

/// Deaths, counted twice over: since anybody last looked, which the weather
/// clears when it has spent the grace they cost, and since the colony was
/// founded, which nobody clears because a winter is weighed against it.
#[derive(Resource, Default)]
pub struct Toll {
    pub recent: u32,
    pub ever: u64,
}

/// The season the colony is in: how many it started with, and how many it had
/// buried by then.
#[derive(Resource, Default)]
pub struct Reckoning {
    pub began_with: usize,
    pub buried_by_then: u64,
}

/// The air outside the generator's reach at the hour a tick falls on, before
/// the weather is added to it.
pub fn ambient_at(tick: u64) -> f32 {
    climate_at(tick) + diurnal_at(tick)
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

/// Ticks a citizen would spend in the cold walking home from here.
///
/// The two shapes involved do not line up, and that is the whole of the
/// arithmetic. A citizen walks one king move a tick, so the walk is counted in
/// Chebyshev steps; the warm ring is a Euclidean disc, because that is how
/// `heat_at` measures. Along the path `step_toward` takes, both axes close by
/// one a tick until the shorter runs out, so the citizen crosses into the ring
/// either on the diagonal stretch or on the straight one that follows it. Whole
/// ticks, rounded up, because a citizen cannot spend part of one in the cold.
pub fn cost_of_getting_home(at: IVec2, air: &Air) -> f32 {
    air.fires
        .iter()
        .map(|fire| cold_walk(at, fire.at, fire.reach(air.ambient)))
        .fold(f32::INFINITY, f32::min)
        // With nothing lit anywhere there is no walk that ends in warmth, and
        // the mark this feeds is above the ceiling for as long as that is true,
        // which is the honest reading of a colony whose fire has gone out.
        .min(cold_walk(at, CENTER, 0.0))
}

/// Ticks in the cold walking from one cell to a fire with this reach.
fn cold_walk(at: IVec2, to: IVec2, reach: f32) -> f32 {
    let away = (at - to).abs();
    let far = away.max_element() as f32;
    let near = away.min_element() as f32;
    if far * far + near * near <= reach * reach {
        return 0.0;
    }
    // The straight stretch, once the shorter axis has run out.
    let straight = far - reach;
    if straight >= near {
        return straight.ceil();
    }
    // The diagonal stretch: both axes close together, so the crossing is the
    // smaller root of `(far - k)^2 + (near - k)^2 = reach^2`.
    let sum = far + near;
    let gap = far - near;
    ((sum - (2.0 * reach * reach - gap * gap).max(0.0).sqrt()) / 2.0).ceil()
}

/// The margin a citizen keeps on top of the walk home.
///
/// There is one base and nothing moves it yet. Under ADR 0017 caution above the
/// base comes only from the tradition cards a citizen carries, so that every
/// cautious citizen is cautious for a reason a watcher can read off them; the
/// card an expedition death leaves behind is the first one there will be, and
/// it arrives with the culture layer rather than here. This is where it reaches
/// in.
pub fn caution_margin(_citizen: &Citizen) -> f32 {
    CAUTION_BASE
}

pub fn generator_output(fuel: u32, upgrades: usize) -> f32 {
    let ceiling = GENERATOR_HEAT + upgrades as f32 * UPGRADE_HEAT;
    (fuel as f32 / FULL_BURN_FUEL as f32).min(1.0) * ceiling
}

/// A place the colony keeps warm. The hearth is one, and the only one today.
#[derive(Debug, Clone, Copy)]
pub struct Fire {
    pub at: IVec2,
    pub output: f32,
}

impl Fire {
    pub fn heat_at(self, p: IVec2, ambient: f32) -> f32 {
        let away = p.as_vec2().distance(self.at.as_vec2());
        (self.output - away * HEAT_FALLOFF).max(0.0) + ambient
    }

    /// How far out this fire's heat still clears the air: the last ring of
    /// cells at which a citizen standing by it stops losing warmth.
    pub fn reach(self, ambient: f32) -> f32 {
        ((self.output + ambient) / HEAT_FALLOFF).max(0.0)
    }
}

/// The air the colony is standing in: every fire it is keeping, and how cold it
/// is outside all of them. The two always travel together.
///
/// A single fire is a closed form evaluable at any coordinate in any order,
/// which is what an unbounded world can afford; a second one costs a walk over
/// the list. That is the price of a fire that is not the hearth, and it is why
/// the list is meant to stay short.
#[derive(Resource, Debug, Clone, Default)]
pub struct Air {
    pub fires: Vec<Fire>,
    pub ambient: f32,
}

impl Air {
    /// The warmest any of the colony's fires makes this square. Warmth does not
    /// add up: standing between two fires is as good as standing by the better
    /// of them and no better.
    pub fn heat_at(&self, p: IVec2) -> f32 {
        self.fires
            .iter()
            .map(|fire| fire.heat_at(p, self.ambient))
            .fold(self.ambient, f32::max)
    }

    /// The nearest warmth worth walking to: whichever fire still heats its own
    /// square and is fewest steps away, and the citizen's own roof if none does.
    pub fn warmth_target(&self, home: IVec2, from: IVec2) -> IVec2 {
        self.fires
            .iter()
            .filter(|fire| fire.heat_at(fire.at, self.ambient) > 0.0)
            .min_by_key(|fire| (fire.at - from).abs().max_element())
            .map_or(home, |fire| fire.at)
    }
}

pub fn step_toward(from: IVec2, to: IVec2) -> IVec2 {
    from + (to - from).signum()
}

/// The most urgent thing a citizen could be doing. A need that kills outranks
/// the load on their back; tiredness does not.
/// How long until tonight's performance, in ticks, which is also in cells: a
/// citizen covers one of each per tick.
pub fn ticks_until_performance(tick: u64) -> u64 {
    let start = PERFORMANCE_HOUR * TICKS_PER_HOUR;
    let now = tick % ticks_per_day();
    if now < start {
        start - now
    } else {
        ticks_per_day() - now + start
    }
}

/// The stage this citizen should be walking to, if the walk is worth starting
/// now. Nobody leaves early: the walk begins when it is as long as the time
/// left, so attending costs the colony a walk it was going to make anyway
/// rather than an afternoon.
pub fn stage_to_attend(stages: &[IVec2], at: IVec2, tick: u64) -> Option<IVec2> {
    let left = ticks_until_performance(tick);
    if left > PERFORMANCE_CALL {
        return None;
    }
    stages
        .iter()
        .map(|stage| ((*stage - at).abs().max_element() as u64, *stage))
        .filter(|(walk, _)| *walk <= PERFORMANCE_CALL && *walk >= left)
        .min_by_key(|(walk, stage)| (*walk, stage.x, stage.y))
        .map(|(_, stage)| stage)
}

pub fn choose_duty(
    needs: &Needs,
    carrying: Option<Cargo>,
    grown: bool,
    marks: &[Marks; NEED_COUNT],
    stage: Option<IVec2>,
) -> Duty {
    for kind in needs.pressing_by_urgency(marks) {
        match kind {
            NeedKind::Warmth => return Duty::WarmUp,
            NeedKind::Food => return Duty::Eat,
            NeedKind::Rest if carrying.is_none() => return Duty::Rest,
            NeedKind::Rest => {}
            // The one need that takes somebody else's hour, so it is the one
            // need with nowhere to go until somebody is offering. A citizen
            // with a load delivers it first: a stage is not worth dropping
            // what the colony is waiting on.
            NeedKind::Recreation if grown && carrying.is_none() && stage.is_some() => {
                return Duty::Attend;
            }
            NeedKind::Recreation => {}
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

/// The places a citizen could be walking to this tick, gathered so that picking
/// one reads as a choice between errands rather than as an argument list.
#[derive(Debug, Clone, Copy)]
pub struct Errands {
    /// Where a load already in hand is wanted.
    pub drop_off: IVec2,
    /// The patch they are working, if any is left on it.
    pub source: Option<IVec2>,
    /// Tonight's performance, if it is worth setting out for yet.
    pub stage: Option<IVec2>,
}

/// Where a citizen walks this tick. `source` is the patch they are working, if
/// any is left, and `drop_off` is wherever their load is wanted.
pub fn duty_target(
    duty: Duty,
    trade: Trade,
    air: &Air,
    at: IVec2,
    home: IVec2,
    to: Errands,
) -> IVec2 {
    match duty {
        Duty::WarmUp => air.warmth_target(home, at),
        Duty::Eat => CENTER,
        Duty::Deliver => to.drop_off,
        Duty::Rest => home,
        // Nobody holds this duty without a stage to hold it for, so the warm
        // ground is a fallback that never fires rather than a second rule.
        Duty::Attend => to.stage.unwrap_or_else(|| air.warmth_target(home, at)),
        // An entertainer's work is the evening, so their day is spent standing
        // where it will happen. The warmest cell is the stage because a cold
        // audience does not gather and a cold performance does not land, and
        // the cost of the trade is exactly this: a pair of hands that fetches
        // nothing.
        Duty::Gather if trade == Trade::Entertainer => air.warmth_target(home, at),
        // With the patches stripped there is no work left, only warmth to find.
        Duty::Gather => to.source.unwrap_or_else(|| air.warmth_target(home, at)),
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
///
/// Own kind at any reach still beats the other kind close by, which is the rule
/// the colony ran on before it had a horizon.
pub fn gather_source(
    patches: &mut Patches,
    want: Cargo,
    from: IVec2,
    take_other: bool,
) -> Option<(IVec2, Cargo)> {
    if let Some(pos) = patches.nearest(want, from, SEARCH_LIMIT) {
        return Some((pos, want));
    }
    // Falling back on a store the colony already has more than enough of buys
    // nothing and keeps a citizen out in the cold to do it.
    if !take_other {
        return None;
    }
    patches
        .nearest(want.other(), from, SEARCH_LIMIT)
        .map(|pos| (pos, want.other()))
}

/// How many hands a colony of this size spares for looking. One as soon as
/// there is a colony at all, and a share of it after that.
pub fn scouts_wanted(hands: usize) -> usize {
    if hands == 0 {
        return 0;
    }
    ((hands as f32 * SCOUT_SHARE).round() as usize).max(1)
}

/// One leg of a scout's walk: mostly short, occasionally very long.
///
/// The step is drawn by inverting a tail that goes as one over the square of
/// the distance, so the share of legs longer than some reach is the shortest
/// leg over that reach. That exponent is the search-theory answer for sparse
/// targets worth revisiting, and it is the mathematics that is adopted here --
/// the animal evidence it was once argued from does not carry, since the
/// albatross tracks behind it turned out to be a sensor artefact.
pub fn scout_step(seed: u64, salt: u64) -> i32 {
    let draw = noise(seed, salt).max(f32::MIN_POSITIVE);
    ((SCOUT_STEP_MIN as f32 / draw) as i32).clamp(SCOUT_STEP_MIN, SCOUT_STEP_MAX)
}

/// Where one leg of a scout's walk ends: a heavy-tailed step in a direction
/// drawn from the same seed, so a run replays exactly.
pub fn scout_target(from: IVec2, seed: u64, salt: u64) -> IVec2 {
    let step = scout_step(seed, salt) as f32;
    let angle = noise(seed, salt.wrapping_add(SCOUT_TURN_SALT)) * std::f32::consts::TAU;
    from + IVec2::new(
        (angle.cos() * step).round() as i32,
        (angle.sin() * step).round() as i32,
    )
}

pub fn chunk_of(cell: IVec2) -> IVec2 {
    IVec2::new(cell.x.div_euclid(CHUNK), cell.y.div_euclid(CHUNK))
}

fn coords(at: IVec2) -> u64 {
    ((at.x as u32 as u64) << 32) | at.y as u32 as u64
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// The generated axis of the world: value noise on a coarse lattice, which is
/// what it is rather than Perlin -- there are no gradients here, only hashed
/// corners interpolated between. A per-cell hash has no continuity at all, and
/// a field is exactly the thing that needs neighbouring cells to resemble one
/// another.
pub fn field_at(world: u64, cell: IVec2) -> f32 {
    let corner = IVec2::new(cell.x.div_euclid(LATTICE), cell.y.div_euclid(LATTICE));
    let across = smooth(cell.x.rem_euclid(LATTICE) as f32 / LATTICE as f32);
    let down = smooth(cell.y.rem_euclid(LATTICE) as f32 / LATTICE as f32);
    let at = |dx: i32, dy: i32| {
        noise(
            world,
            FIELD_SALT.wrapping_add(coords(corner + IVec2::new(dx, dy))),
        )
    };
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * across;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * across;
    top + (bottom - top) * down
}

/// How much better a patch is for being this far out. Flat inside the old rim,
/// so nothing about the early game changes, then rising until it stops.
pub fn richness_at(distance: i32) -> f32 {
    let out = (distance - PATCH_RADIUS).max(0) as f32;
    1.0 + (out / RICHNESS_RUN as f32).min(RICHNESS_CAP)
}

/// What a chunk holds.
///
/// FORBIDDEN, and this is the door it would come in by: nothing here may read
/// what a neighbouring chunk holds. A feature written across a border forces the
/// neighbour to exist, which forces its neighbour, and the cascade is a named
/// failure in the games that shipped it.
///
/// Reading past the border is not the same as reading the neighbour. `field_at`
/// is keyed to absolute coordinates and steps a lattice whose corners fall
/// wherever they fall, which is exactly what makes the field continuous across a
/// border without either side of it having to exist. The line is between a pure
/// function of the seed and a position, which may be evaluated anywhere in any
/// order, and a look at another chunk's contents, which may not.
///
/// Nothing here may draw from `Lineage` either. It hands out sequential numbers
/// and is order-dependent by construction, so a world that asked it for
/// anything would stop replaying the moment a chunk was realised out of order.
pub fn chunk_patches(world: u64, chunk: IVec2) -> Vec<Patch> {
    let seed = mix(world, CHUNK_SALT.wrapping_add(coords(chunk)));
    let corner = chunk * CHUNK;
    let mut patches: Vec<Patch> = Vec::with_capacity(PATCHES_PER_CHUNK);
    for index in 0..PATCHES_PER_CHUNK as u64 {
        let pos = corner
            + IVec2::new(
                (noise(seed, index * 4) * CHUNK as f32) as i32,
                (noise(seed, index * 4 + 1) * CHUNK as f32) as i32,
            );
        // The ground the colony builds on is cleared, measured the way the rings
        // that stood there were.
        if pos.as_vec2().distance(CENTER.as_vec2()) <= PLOT_MAX_RADIUS as f32 {
            continue;
        }
        if patches.iter().any(|patch| patch.pos == pos) {
            continue;
        }
        let kind = if noise(seed, index * 4 + 2) < WOOD_SHARE {
            Cargo::Wood
        } else {
            Cargo::Food
        };
        let base = match kind {
            Cargo::Wood => WOOD_PER_CELL,
            Cargo::Food => FOOD_PER_CELL,
        } as f32;
        let out = (pos - CENTER).abs().max_element();
        let worth = 1.0 - FIELD_SWING / 2.0 + field_at(world, pos) * FIELD_SWING;
        let cap = (base * richness_at(out) * worth).round().max(1.0) as u32;
        patches.push(Patch {
            pos,
            kind,
            amount: cap,
            cap,
        });
    }
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
pub fn delivery_target(
    cargo: Cargo,
    diverting: bool,
    site: Option<IVec2>,
    claimed: Option<IVec2>,
) -> IVec2 {
    if cargo == Cargo::Wood {
        if let Some(pos) = site
            && diverting
        {
            return pos;
        }
        // A shed short of wood claims one hauler, the one nearest it, and takes
        // their load ahead of the hearth. One each is what makes it a tax that
        // grows with the length of a chain rather than a conscription that
        // empties the hearth the moment a single post stands. It is paid out of
        // the same surplus that pays for building and out of nothing else.
        if diverting && let Some(post) = claimed {
            return post;
        }
    }
    CENTER
}

/// The building that answers a given need, where one does.
///
/// Three of the four have a remedy the colony can put up. The fourth is
/// answered by somebody's time instead, so there is nothing for a bored citizen
/// to ask the ballot for -- which is why this hands back nothing rather than
/// picking a building that would not help.
pub fn building_for(kind: NeedKind) -> Option<Building> {
    match kind {
        NeedKind::Warmth => Some(Building::GeneratorUpgrade),
        NeedKind::Rest => Some(Building::House),
        NeedKind::Food => Some(Building::HuntersHut),
        NeedKind::Recreation => None,
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
    // Only needs with a remedy are on this tier at all: a citizen suffering
    // something the colony cannot build its way out of has nothing to ask it
    // for, and asking for the next-best thing would be a vote nobody cast.
    let mut choice = None;
    for kind in NEEDS {
        if building_for(kind).is_none() {
            continue;
        }
        if choice.is_none_or(|best| needs.get(kind).burden > needs.get(best).burden) {
            choice = Some(kind);
        }
    }
    if let Some(choice) = choice
        && needs.get(choice).burden > 0.0
    {
        return building_for(choice);
    }
    // Nothing went unanswered, so the colony is holding for this citizen and
    // the question is no longer survival but waste. The two are never compared:
    // a starving colony votes food whatever its haulers are walking, and a
    // comfortable one is free to spend the ballot on efficiency.
    if needs.detour_burden() > 0.0 {
        // A bigger hearth answers hours lost all around it; a nearer fire
        // answers hours lost in one direction, and only the second is worth
        // moving warmth for.
        let concentrated = needs
            .detour_middle()
            .is_some_and(|at| (at - CENTER).abs().max_element() > BOILER_REACHES);
        return Some(if concentrated {
            Building::Waystation
        } else {
            Building::GeneratorUpgrade
        });
    }
    None
}

/// Where a waystation goes: the middle of the walks that asked for one, each
/// citizen weighing as many hours as they lost. Choosing the spot is arithmetic
/// and not a decision -- the decision was the vote.
pub fn waystation_site(people: &[Needs], air: &Air) -> Option<IVec2> {
    let mut hours = 0;
    let mut sum = IVec2::ZERO;
    for needs in people {
        if vote_of(needs) != Some(Building::Waystation) {
            continue;
        }
        let Some(middle) = needs.detour_middle() else {
            continue;
        };
        hours += needs.detour;
        sum += (middle - CENTER) * needs.detour as i32;
    }
    if hours == 0 {
        return None;
    }
    // Where the walks concentrate is where a post is wanted; whether it can be
    // supplied is another question, and the second one decides. A post further
    // out than a hauler can reach and return from is a post that never burns,
    // so the site is drawn back along the line until it is one that can be fed.
    let mut site = CENTER + sum / hours as i32;
    while site != CENTER && cost_of_getting_home(site, air) > WAYSTATION_STEP {
        site = step_toward(site, CENTER);
    }
    (site != CENTER).then_some(site)
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
    NEEDS
        .into_iter()
        .filter(|kind| kind.is_survival())
        .all(|kind| shares[kind as usize] >= GROWTH_SHARE)
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

/// How the founding party divides itself up. The labourer pool is set aside
/// first, because a colony with nobody spare has no way to fill a vacancy; the
/// rest go mostly to the treelines, because the fire is most of what a colony
/// spends and nobody quits a trade once they have it.
///
/// Nobody is founded to the evening. The rule the trade lives under is that an
/// hour spent amusing is spent on a plateau and never in a winter, and the
/// founding party is the leanest the colony will ever be: it has stores for a
/// season, no houses beyond its own, and no idea yet where the wood is. The
/// first entertainer is appointed once the colony has earned one.
pub fn founding_trade(index: usize) -> Trade {
    let spare = laborers_wanted(CITIZENS);
    if index < spare {
        Trade::Laborer
    } else if (index - spare).is_multiple_of(HUNTERS_ONE_IN) {
        Trade::Hunter
    } else {
        Trade::Woodcutter
    }
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
        // The upbringing is the one place a citizen's stats come from, so their
        // span is read off it rather than derived alongside it.
        let upbringing = Upbringing::grown(seed);
        let lifespan = lifespan_of(
            seed,
            upbringing.stats().of(Stat::Hardiness),
            upbringing.prosperity(),
        );
        commands.spawn((
            Pos(ring_pos(START_RING, angle)),
            Citizen {
                needs: Needs::founder(i, CITIZENS),
                upbringing,
                acclimated: 0.0,
                watched: 0.0,
                banked: 0.0,
                load: 0,
                trade: founding_trade(i),
                experience: [0.0; TRADE_COUNT],
                age: founder_age(i, CITIZENS),
                lifespan,
                seed,
                home,
                carrying: None,
                hauling: Cargo::Wood,
                scouting: None,
                mood: MOOD_BASE,
                held: [false; THOUGHT_COUNT],
                hardship: 0.0,
                spared_until: 0,
                cheered_until: 0,
            },
        ));
    }

    commands.insert_resource(Patches::new(WORLD_SEED));
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
    let households: Vec<(IVec2, Trade, f32, f32)> = citizens
        .iter()
        .filter(|citizen| is_adult(citizen.age))
        .map(|citizen| {
            (
                citizen.home,
                citizen.trade,
                citizen.experience[citizen.trade as usize],
                citizen.age,
            )
        })
        .collect();
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
        if citizen.upbringing.settle_due(age) {
            // The childhood is over, so the span stops being the guess it was
            // made with and becomes what the colony actually raised.
            citizen.lifespan = lifespan_of(
                citizen.seed,
                citizen.upbringing.stats().of(Stat::Hardiness),
                citizen.upbringing.prosperity(),
            );
            // And they take up the house's trade, with some of the practice the
            // head of it had in the work.
            if let Some((trade, practice)) = household_head(&households, citizen.home) {
                citizen.trade = trade;
                citizen.experience[trade as usize] = inherited_experience(practice);
            }
        }
        citizen.upbringing.catch_up(age, warmth, food);
    }
}

/// The middle of a colony, which is what one citizen's stat is read against.
/// Sorts in place because the caller owns the scratch and nobody else wants it.
/// The middle of a set, and what to say when the set is empty. The second is a
/// parameter because these are asked about scales that do not share a zero: a
/// stat runs nought to one and a mood runs nought to a hundred, and a neutral
/// borrowed from the wrong one reads as a real reading of a colony that is not
/// there any more.
pub fn median(values: &mut [f32], empty: f32) -> f32 {
    if values.is_empty() {
        return empty;
    }
    values.sort_by(f32::total_cmp);
    values[values.len() / 2]
}

pub fn advance_calendar(tick: Res<Tick>, mut calendar: ResMut<Calendar>) {
    *calendar = calendar_at(tick.0);
}

/// What the colony gives up on. Once a year it looks at whoever has been gone
/// long enough and raises a slab for them, if it can spare the wood.
pub fn raise_memorials(
    tick: Res<Tick>,
    mut missing: ResMut<Missing>,
    mut generator: ResMut<Generator>,
) {
    if !tick.0.is_multiple_of(ticks_per_season()) {
        return;
    }
    missing.raise_memorials(tick.0, &mut generator.fuel);
}

/// Every waystation burns its own shed down on its own clock, whether or not
/// anybody is standing at it. Keeping one lit is a standing cost.
pub fn burn_caches(tick: Res<Tick>, mut posts: Query<&mut Cache>) {
    if !tick.0.is_multiple_of(WAYSTATION_BURN_EVERY) {
        return;
    }
    for mut cache in &mut posts {
        cache.0 = cache.0.saturating_sub(1);
    }
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
    stores: Stores,
    citizens: Query<&Citizen>,
    posts: Query<(&Pos, &Cache)>,
    mut sky: Sky,
) {
    let (weather, toll) = (&mut sky.weather, &mut sky.toll);
    // Grace is earned and spent by the tick, because a death happens on one.
    weather.adaptation = adaptation_step(weather.adaptation, toll.recent > 0);
    toll.recent = 0;

    // The weather moves once a day and the climate and the hour move with every
    // tick, so the front and the spell are stepped on the day boundary and
    // carried across the hours in between.
    let day = tick.0 / ticks_per_day();
    let population = citizens.iter().count();
    let budget = severity_budget(
        population,
        stores.generator.fuel,
        stores.granary.food,
        tick.0 / ticks_per_year() + 1,
        weather.adaptation,
    );
    while weather.day < day {
        weather.day += 1;
        weather.front = front_step(weather.front, WORLD_SEED, weather.day);
        if weather
            .spell
            .is_none_or(|spell| weather.day >= spell.began + spell.days)
        {
            weather.spell = spell_due(budget, WORLD_SEED, weather.day);
        }
    }
    let spelling = weather.spell.map_or(0.0, |spell| spell.air_on(day));
    weather.word = weather_word(weather.spell.as_ref(), day);
    let mut fires = vec![Fire {
        at: CENTER,
        output: generator_output(stores.generator.fuel, built.of(Building::GeneratorUpgrade)),
    }];
    // A waystation with an empty shed is a place to stand and nothing more.
    fires.extend(
        posts
            .iter()
            .filter(|(_, cache)| cache.0 > 0)
            .map(|(pos, _)| Fire {
                at: pos.0,
                output: WAYSTATION_HEAT,
            }),
    );
    *sky.air = Air {
        fires,
        ambient: ambient_at(tick.0) + weather.front + spelling,
    };
}

/// A day's growing back, in the seasons that allow it.
pub fn regrow_patches(tick: Res<Tick>, calendar: Res<Calendar>, mut patches: ResMut<Patches>) {
    if !tick.0.is_multiple_of(ticks_per_day()) {
        return;
    }
    patches.regrow(is_growing_season(calendar.season));
}

/// Who the colony can spare for looking, and where they set off to.
///
/// Being idle is the whole qualification: a citizen the survival tier of the
/// ballot has nothing to say about is one nothing has been failing, and that is
/// exactly the citizen whose day the colony is not already spending. A tour
/// lasts a season, because the question of who can be spared is worth asking
/// again by then.
pub fn send_scouts(
    tick: Res<Tick>,
    air: Res<Air>,
    missing: Res<Missing>,
    mut citizens: Query<(&Pos, &mut Citizen)>,
) {
    if !tick.0.is_multiple_of(ticks_per_season()) {
        return;
    }
    let hands = citizens
        .iter()
        .filter(|(_, citizen)| is_adult(citizen.age))
        .count();
    let wanted = scouts_wanted(hands);
    let mut spare: Vec<(u32, u64)> = citizens
        .iter()
        .filter(|(_, citizen)| is_adult(citizen.age) && citizen.needs.nothing_failed())
        .map(|(_, citizen)| {
            (
                (noise(citizen.seed, SCOUT_PICK_SALT.wrapping_add(tick.0)) * u32::MAX as f32)
                    as u32,
                citizen.seed,
            )
        })
        .collect();
    // Drawn rather than ordered, so that the same few are not sent every season
    // and the colony still decides the same way in every run.
    spare.sort_unstable();
    spare.truncate(wanted);
    // A leg is drawn from the furthest fire the colony keeps rather than from
    // wherever the scout is standing, because everything past that fire is the
    // reach the chain was built to buy. The walk out to it is not free: a
    // brazier holds a ring of about five cells, so most of the ground between
    // one fire and the next is cold, and a scout arrives having already spent
    // it. That gap is why the chain does not yet extend anybody's reach, and it
    // is measured rather than supposed.
    let setting_out = air
        .fires
        .iter()
        .filter(|fire| fire.heat_at(fire.at, air.ambient) > 0.0)
        .max_by_key(|fire| (fire.at - CENTER).abs().max_element())
        .map_or(CENTER, |fire| fire.at);
    // A walk with somebody at the end of it beats a walk drawn from a hat. The
    // colony spends the same scouting season either way, which is what §14 means
    // by a recovery costing a second citizen the same trip.
    let looking_for = missing.nearest_to(setting_out);
    for (_, mut citizen) in &mut citizens {
        citizen.scouting = spare
            .iter()
            .any(|(_, seed)| *seed == citizen.seed)
            .then(|| {
                looking_for.unwrap_or_else(|| scout_target(setting_out, citizen.seed, tick.0))
            });
    }
}

/// How the last performance went, so that a watcher can see the thing the
/// entertainer's hour bought.
#[derive(Resource)]
pub struct Revels(pub Outcome);

/// Where tonight's performances will be. Published before anybody decides what
/// to do with the rest of their day, which is the whole of what makes an
/// audience assemble rather than happen to be standing there.
#[derive(Resource, Default)]
pub struct Stages(pub Vec<IVec2>);

/// Entertainers hold the warm ground, so the call is where they are standing
/// rather than a booking made in advance.
pub fn call_the_evening(mut stages: ResMut<Stages>, citizens: Query<(&Pos, &Citizen)>) {
    stages.0.clear();
    stages.0.extend(
        citizens
            .iter()
            .filter(|(_, citizen)| citizen.trade == Trade::Entertainer && is_adult(citizen.age))
            .map(|(pos, _)| pos.0),
    );
}

impl Default for Revels {
    fn default() -> Self {
        Self(Outcome::Boring)
    }
}

/// The evening's entertainment.
///
/// An entertainer performs where they are standing, once a day, and the
/// audience is whoever is close enough and warm enough to be listening rather
/// than working. Everything about how it goes comes off the named terms, and
/// what it pays is boredom lifted and -- when it went well -- a few days of
/// somebody remembering it.
pub fn perform(
    tick: Res<Tick>,
    air: Res<Air>,
    mut revels: ResMut<Revels>,
    mut citizens: Query<(&Pos, &mut Citizen)>,
) {
    if tick.0 % ticks_per_day() != PERFORMANCE_HOUR * TICKS_PER_HOUR {
        return;
    }
    let stages: Vec<(IVec2, f32, u64)> = citizens
        .iter()
        .filter(|(_, citizen)| citizen.trade == Trade::Entertainer && is_adult(citizen.age))
        .map(|(pos, citizen)| {
            let raised = citizen.upbringing.stats().of(trade_stat(citizen.trade));
            let fit = trade_fit(raised, citizen.experience[citizen.trade as usize]);
            (
                pos.0,
                (raised * fit / FIT_CEILING).clamp(0.0, 1.0),
                citizen.seed,
            )
        })
        .collect();
    for (stage, performer, seed) in stages {
        let warmth = air.heat_at(stage);
        let audience: Vec<IVec2> = citizens
            .iter()
            .filter(|(pos, _)| {
                (pos.0 - stage).abs().max_element() <= PERFORMANCE_REACH
                    && air.heat_at(pos.0) >= 0.0
            })
            .map(|(pos, _)| pos.0)
            .collect();
        // The tradition term is nothing until a colony has traditions.
        let quality = performance_quality(audience.len(), warmth, performer, 0.0);
        let outcome = performance_outcome(quality, WORLD_SEED, tick.0 ^ seed);
        let cheered_until = tick.0 + outcome.mood_days() * ticks_per_day();
        revels.0 = outcome;
        for (pos, mut citizen) in &mut citizens {
            if (pos.0 - stage).abs().max_element() > PERFORMANCE_REACH || air.heat_at(pos.0) < 0.0 {
                continue;
            }
            citizen.needs.amuse(outcome.worth());
            if outcome.went_well() {
                citizen.cheered_until = citizen.cheered_until.max(cheered_until);
            }
        }
    }
}

/// What a winter cost, weighed when it is over.
///
/// A colony that has just buried a share of itself measures the next seasons
/// against a worse year than the one it had, so the same cold and the same
/// hunger weigh less on whoever came through. It touches nothing anybody acts
/// on -- lowering the mark somebody sets out for warmth at would send them out
/// later and kill them -- only what a day costs them to live through.
///
/// It is weighed every season and not only after a winter, which is a departure
/// from the letter of ADR 0013 and was forced by measurement: across five worlds
/// the worst winter took five per cent of a colony and the worst autumn took
/// twenty-seven, because the cold arrives while the colony is still working to
/// a summer pattern and by winter whoever could not bear it is already gone.
/// Keyed to winters the rule never once fired in nearly four hundred years.
pub fn weigh_the_season(
    tick: Res<Tick>,
    mut reckoning: ResMut<Reckoning>,
    toll: Res<Toll>,
    mut citizens: Query<&mut Citizen>,
) {
    if !tick.0.is_multiple_of(ticks_per_season()) {
        return;
    }
    let buried = toll.ever.saturating_sub(reckoning.buried_by_then);
    if season_broke_them(reckoning.began_with, buried) {
        let until = spared_until(tick.0);
        for mut citizen in &mut citizens {
            citizen.spared_until = until;
        }
    }
    reckoning.began_with = citizens.iter().count();
    reckoning.buried_by_then = toll.ever;
}

/// The colony keeps only the world it is standing in. Everything else goes back
/// to being a seed and the cuts it remembers -- and those are the half that does
/// not shrink when the colony moves on, which is why the ceiling counts them.
pub fn forget_far_world(
    tick: Res<Tick>,
    mut patches: ResMut<Patches>,
    citizens: Query<&Pos, With<Citizen>>,
) {
    if !tick.0.is_multiple_of(ticks_per_season()) {
        return;
    }
    let mut homes = vec![CENTER];
    homes.extend(citizens.iter().map(|pos| pos.0));
    patches.forget_beyond(&homes, FORGET_BEYOND);
}

/// One reading a day of what the colony holds per head, kept a season deep.
pub fn record_trend(
    tick: Res<Tick>,
    stores: Stores,
    citizens: Query<&Citizen>,
    mut trend: ResMut<Trend>,
    mut flow: ResMut<Flow>,
    mut postings: ResMut<Postings>,
) {
    if !tick.0.is_multiple_of(ticks_per_day()) {
        return;
    }
    let population = citizens.iter().count();
    postings.note(
        Cargo::Wood,
        stock_share(stores.generator.fuel, FUEL_PER_CITIZEN, population) < 1.0,
    );
    postings.note(
        Cargo::Food,
        stock_share(stores.granary.food, FOOD_PER_CITIZEN, population) < 1.0,
    );
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

/// Filling vacancies. Nobody quits, so the only hands going spare are the ones
/// the colony holds back, and a vacancy goes to one of the best few of them
/// rather than to the best -- which is what keeps the wrong person occasionally
/// getting the right job, and the stories that come with it.
pub fn assign_trades(
    tick: Res<Tick>,
    stores: Stores,
    mut patches: ResMut<Patches>,
    postings: Res<Postings>,
    mayor: Res<Mayor>,
    mut citizens: Query<(Entity, &Pos, &mut Citizen)>,
) {
    if !tick.0.is_multiple_of(ticks_per_season()) {
        return;
    }
    let hands = citizens
        .iter()
        .filter(|(_, _, citizen)| is_adult(citizen.age))
        .count();
    let short = if stock_share(stores.granary.food, FOOD_PER_CITIZEN, hands)
        < stock_share(stores.generator.fuel, FUEL_PER_CITIZEN, hands)
    {
        Cargo::Food
    } else {
        Cargo::Wood
    };
    // A vacancy goes to the entertainer only while the colony is not short of
    // anything, which is ADR 0011's crisis rule expressed where the hours are
    // actually handed out: an hour spent amusing is an hour not spent on wood,
    // so it is spent on a plateau and never in a winter.
    let comfortable = stock_share(stores.granary.food, FOOD_PER_CITIZEN, hands) >= 1.0
        && stock_share(stores.generator.fuel, FUEL_PER_CITIZEN, hands) >= 1.0;
    let players = citizens
        .iter()
        .filter(|(_, _, citizen)| is_adult(citizen.age) && citizen.trade == Trade::Entertainer)
        .count();
    let trade = if comfortable && players < entertainers_wanted(hands) {
        Trade::Entertainer
    } else {
        trade_for(short)
    };
    let stale = posting_is_stale(postings.days_short(short));

    let mut laborers = 0usize;
    let mut spare: Vec<(Entity, IVec2, f32)> = Vec::new();
    for (entity, pos, citizen) in &citizens {
        if !is_adult(citizen.age) {
            continue;
        }
        if citizen.trade == Trade::Laborer {
            laborers += 1;
        }
        if may_be_taken(citizen.trade, trade, stale) {
            spare.push((entity, pos.0, citizen.experience[trade as usize]));
        }
    }
    // Spare hands are what the colony holds back; a stale posting is allowed to
    // eat into a trade instead, which is the only way a first guess at a
    // workforce ever gets corrected.
    let vacancies = if stale {
        spare.len().min(1)
    } else {
        laborers.saturating_sub(laborers_wanted(hands))
    };
    if vacancies == 0 {
        return;
    }
    let bias = mayor.trade_bias[trade as usize];

    for filled in 0..vacancies {
        let scored: Vec<(usize, f32)> = spare
            .iter()
            .enumerate()
            .map(|(index, (_, at, experience))| {
                let walk = gather_source(&mut patches, short, *at, false)
                    .map_or(WALK_UNFOUND, |(cell, _)| (cell - *at).abs().max_element());
                (index, assignment_score(walk, *experience, bias))
            })
            .collect();
        let Some(taken) = pick_from_top(&scored, noise(tick.0, filled as u64)) else {
            break;
        };
        let (entity, ..) = spare.swap_remove(taken);
        if let Ok((_, _, mut citizen)) = citizens.get_mut(entity) {
            citizen.trade = trade;
        }
    }
}

/// Finishes the project in progress, or opens the next one on a free plot once
/// the colony has timber to spare.
pub fn construction(
    mut commands: Commands,
    mut council: Council,
    calendar: Res<Calendar>,
    generator: Res<Generator>,
    air: Res<Air>,
    structures: Query<(&Pos, &Structure)>,
    mut citizens: Query<&mut Citizen>,
) {
    council.construction.diverting =
        update_diverting(council.construction.diverting, generator.fuel);

    if let Some(site) = &council.construction.site {
        if site.delivered >= site.building.rules().cost {
            let mut raised = commands.spawn((Pos(site.pos), Structure(site.building)));
            if site.building == Building::Waystation {
                raised.insert(Cache(0));
            }
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
    // A waystation goes where the walks that asked for it were walked; anything
    // else goes on the next free plot. Choosing the spot is arithmetic either
    // way -- the decision was the vote.
    let where_to = if building == Building::Waystation {
        waystation_site(&people, &air).filter(|pos| !taken.contains(pos))
    } else {
        next_plot(&taken)
    };
    if let Some(pos) = where_to {
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
        let upbringing = Upbringing::born(seed);
        // A guess until the childhood that will settle it has happened.
        let lifespan = lifespan_of(
            seed,
            upbringing.stats().of(Stat::Hardiness),
            upbringing.prosperity(),
        );
        commands.spawn((
            Pos(CENTER),
            Citizen {
                needs: Needs::newcomer(),
                upbringing,
                acclimated: 0.0,
                watched: 0.0,
                banked: 0.0,
                load: 0,
                // A child is nobody's tradesman until they are grown.
                trade: Trade::Laborer,
                experience: [0.0; TRADE_COUNT],
                age: 0.0,
                lifespan,
                seed,
                home,
                carrying: None,
                hauling: Cargo::Wood,
                scouting: None,
                mood: MOOD_BASE,
                held: [false; THOUGHT_COUNT],
                hardship: 0.0,
                spared_until: 0,
                cheered_until: 0,
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
    let air = &*air;
    // Every shed short of wood claims the wood-carrier standing nearest it, one
    // each, worked out before anybody moves so that the same colony state always
    // makes the same claims.
    let mut hungry: Vec<IVec2> = colony
        .posts
        .iter()
        .filter(|(_, cache)| cache.0 < WAYSTATION_CACHE)
        .map(|(pos, _)| pos.0)
        .collect();
    hungry.sort_by_key(|post| ((post - CENTER).abs().max_element(), post.x, post.y));
    // The seed breaks the last tie, because two haulers can stand on one cell
    // and the order a query hands them back is not a thing the colony knows.
    let mut carriers: Vec<(Entity, IVec2, u64)> = citizens
        .iter()
        .filter(|(_, _, citizen)| citizen.carrying == Some(Cargo::Wood))
        .map(|(entity, pos, citizen)| (entity, pos.0, citizen.seed))
        .collect();
    let mut claims: Vec<(Entity, IVec2)> = Vec::new();
    for post in hungry {
        let Some(index) = carriers
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, at, seed))| ((post - *at).abs().max_element(), *seed))
            .map(|(index, _)| index)
        else {
            break;
        };
        claims.push((carriers.swap_remove(index).0, post));
    }
    let site_pos = colony.construction.site.as_ref().map(|site| site.pos);
    let population = citizens.iter().count();
    let somebody_missing = colony.missing.count() > 0;
    let plenty = stock_share(colony.generator.fuel, FUEL_PER_CITIZEN, population) >= 1.0
        && stock_share(colony.granary.food, FOOD_PER_CITIZEN, population) >= 1.0;
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
        // What the walk home would cost from where they are standing, which is
        // what warmth's marks are measured against rather than a fixed number.
        let marks = marks_for(cost_of_getting_home(pos.0, air), caution_margin(&citizen));
        let stage = stage_to_attend(&colony.stages.0, pos.0, tick.0);
        let fit = trade_fit(
            citizen.upbringing.stats().of(trade_stat(citizen.trade)),
            citizen.experience[citizen.trade as usize],
        );
        let duty = choose_duty(&citizen.needs, citizen.carrying, grown, &marks, stage);

        // Eating is what makes the food need met this tick, so it happens before
        // the needs are stepped.
        let eating = takes_a_meal(&citizen.needs, at_centre, colony.granary.food);
        if eating {
            colony.granary.food -= FOOD_PER_MEAL;
        }
        let warm = air.heat_at(pos.0) >= 0.0;
        // Keyed by the need rather than positional, so a fourth need cannot be
        // read off the end of a list of three. Recreation is never met by being
        // anywhere: a performance lifts it and nothing else does.
        for kind in NEEDS {
            let met = match kind {
                NeedKind::Warmth => warm,
                NeedKind::Rest => duty == Duty::Rest && at_home,
                NeedKind::Food => eating,
                NeedKind::Recreation => false,
            };
            let scale = if kind == NeedKind::Warmth && at_home {
                SHELTER_DRAIN_FACTOR
            } else {
                1.0
            };
            citizen.needs.step(kind, met, scale, marks[kind as usize]);
        }
        // How they are bearing up, worked out from what is true of them and of
        // the colony rather than from anything this layer had to invent.
        citizen.held = thoughts_of(
            &citizen.needs,
            &marks,
            fit < 1.0,
            somebody_missing,
            plenty,
            tick.0 < citizen.cheered_until,
        );
        citizen.mood = mood_step(
            citizen.mood,
            mood_target(&citizen.held, tick.0 < citizen.spared_until),
            duty == Duty::Rest && at_home,
        );
        citizen.hardship = hardship_step(citizen.hardship, citizen.mood);
        if citizen.needs.spent() {
            colony.missing.take_note(pos.0, tick.0);
            colony.toll.recent += 1;
            colony.toll.ever += 1;
            commands.entity(entity).despawn();
            continue;
        }
        // The snap is the air they are standing in, not how desperate they are.
        // Rolling only once a citizen is already freezing would mean frailty
        // never got to decide anything.
        citizen.acclimated = acclimation_step(citizen.acclimated, !warm);
        if cold_night && !warm {
            let roll = noise(citizen.seed, COLD_SNAP_SALT.wrapping_add(tick.0));
            let resistance = cold_resistance(
                citizen.age,
                citizen.lifespan,
                citizen.upbringing.stats().of(Stat::Hardiness),
                citizen.acclimated,
            );
            if cold_takes(resistance, roll) {
                colony.missing.take_note(pos.0, tick.0);
                colony.toll.recent += 1;
                colony.toll.ever += 1;
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
            let claimed = claims
                .iter()
                .find(|(who, _)| *who == entity)
                .map(|(_, post)| *post);
            let drop_off = delivery_target(cargo, colony.construction.diverting, site_pos, claimed);
            if (pos.0 - drop_off).abs().max_element() <= 1 {
                // What a citizen carries home is the one place what the colony
                // raised in them turns into capacity rather than survival.
                let yield_each = match cargo {
                    Cargo::Wood => 1,
                    Cargo::Food => food_yield(built.of(Building::HuntersHut)),
                };
                let brought = citizen.load * yield_each;
                let to_the_site = match (cargo, colony.construction.site.as_ref()) {
                    (Cargo::Wood, Some(site))
                        if log_goes_to_site(drop_off, site.pos, site.delivered, site.building) =>
                    {
                        (site.building.rules().cost - site.delivered).min(brought)
                    }
                    _ => 0,
                };
                if to_the_site > 0
                    && let Some(site) = colony.construction.site.as_mut()
                {
                    site.delivered += to_the_site;
                }
                // Wood set down at a post goes in that post's shed rather than
                // on the hearth; the hauler walked there instead of home.
                let to_a_shed = (cargo == Cargo::Wood && drop_off != CENTER && to_the_site == 0)
                    .then(|| {
                        colony
                            .posts
                            .iter_mut()
                            .find(|(pos, _)| pos.0 == drop_off)
                            .map(|(_, mut cache)| {
                                let room = WAYSTATION_CACHE.saturating_sub(cache.0);
                                let stowed = brought.min(room);
                                cache.0 += stowed;
                                stowed
                            })
                    })
                    .flatten()
                    .unwrap_or(0);
                match cargo {
                    Cargo::Wood => colony.generator.fuel += brought - to_the_site - to_a_shed,
                    Cargo::Food => colony.granary.food += brought,
                }
                flow.delivered(brought);
                citizen.carrying = None;
                // A tradesman fetches their own kind; a labourer goes wherever
                // the colony is shorter.
                let next = trade_cargo(citizen.trade)
                    .unwrap_or_else(|| haul_choice(citizen.hauling, supply));
                if next != citizen.hauling {
                    inbound[citizen.hauling as usize] -= 1;
                    inbound[next as usize] += 1;
                    citizen.hauling = next;
                }
            }
        }

        let source = gather_source(
            &mut colony.patches,
            citizen.hauling,
            pos.0,
            supply.wants(citizen.hauling.other()),
        );
        if duty == Duty::Gather
            && let Some((cell, kind)) = source
            && cell == pos.0
        {
            // What a citizen lifts is decided here, where it comes out of the
            // ground, so that what leaves the patch is what reaches the store.
            let raised = citizen.upbringing.stats().of(trade_stat(citizen.trade));
            let (wanted, banked) = haul_load(
                effective_stat(raised, focus_of(&citizen.needs), fit),
                citizen.banked,
            );
            let lifted = colony.patches.take(cell, wanted);
            if lifted > 0 {
                citizen.banked = banked;
                citizen.load = lifted;
                citizen.carrying = Some(kind);
            }
        }

        // Handing a load over or picking one up flips this tick's duty; nothing
        // else about the citizen has changed since it was chosen.
        let duty = choose_duty(&citizen.needs, citizen.carrying, grown, &marks, stage);
        // Only a working citizen has working hours for the cold to take.
        if grown {
            citizen.needs.spend(duty == Duty::WarmUp, pos.0);
            // A trade is kept up by working it and goes to rust otherwise.
            let at_work = matches!(duty, Duty::Gather | Duty::Deliver);
            if at_work {
                citizen.watched += per_day(1.0);
            }
            for trade in TRADES {
                let practising = at_work && trade == citizen.trade;
                citizen.experience[trade as usize] =
                    experience_step(citizen.experience[trade as usize], practising);
            }
        }
        let drop_off = citizen.carrying.map_or(CENTER, |cargo| {
            let claimed = claims
                .iter()
                .find(|(who, _)| *who == entity)
                .map(|(_, post)| *post);
            delivery_target(cargo, colony.construction.diverting, site_pos, claimed)
        });
        // A scout is walking to look, not to fetch, so the leg stands in for the
        // patch. Nothing brings them back but the leash: warmth's marks rise
        // with every cell they put between themselves and the fire, and a
        // pressing need outranks the walk.
        let target = match citizen.scouting {
            Some(leg) if duty == Duty::Gather => leg,
            _ => duty_target(
                duty,
                citizen.trade,
                air,
                pos.0,
                citizen.home,
                Errands {
                    drop_off,
                    source: source.map(|(cell, _)| cell),
                    stage,
                },
            ),
        };
        if pos.0 != target {
            pos.0 = step_toward(pos.0, target);
        }
        // Standing somewhere is how the colony comes to know it, and how it
        // finds anybody it lost there. Haulers only ever stand on ground it
        // already knows, so in practice both are the scout's yield.
        if colony.missing.count() > 0 {
            colony.missing.recover(&[pos.0]);
        }
        colony.patches.discover(pos.0);
        if citizen.scouting == Some(pos.0) {
            citizen.scouting = Some(scout_target(pos.0, citizen.seed, tick.0));
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
            fires: vec![Fire {
                at: CENTER,
                output: generator_output(fuel, upgrades),
            }],
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

    /// The marks of a citizen standing at the fire, which are the fixed ones the
    /// colony worked to before warmth learned where it was.
    fn at_the_fire() -> [Marks; NEED_COUNT] {
        marks_for(0.0, CAUTION_BASE)
    }

    fn set(needs: &mut Needs, kind: NeedKind, level: f32, pressing: bool) {
        needs.needs[kind as usize] = Need {
            level,
            pressing,
            burden: shortfall_at(at_the_fire()[kind as usize], level).max(0.0),
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
        assert_eq!(air(FULL_BURN_FUEL, 0).warmth_target(home, CENTER), CENTER);
        assert_eq!(air(0, 0).warmth_target(home, CENTER), home);
    }

    #[test]
    fn shortfall_reads_zero_at_the_high_mark_and_one_at_the_low_one() {
        for kind in NEEDS {
            let rules = kind.rules();
            let mut needs = Needs::newcomer();
            set(&mut needs, kind, rules.high, false);
            assert_eq!(
                needs.shortfall(kind, at_the_fire()[kind as usize]),
                0.0,
                "{kind:?} at the high mark"
            );
            set(&mut needs, kind, rules.low, true);
            assert_eq!(
                needs.shortfall(kind, at_the_fire()[kind as usize]),
                1.0,
                "{kind:?} at the low mark"
            );
            set(&mut needs, kind, 0.0, true);
            assert!(
                needs.shortfall(kind, at_the_fire()[kind as usize]) > 1.0,
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
        let first = needs.shortfall(NEEDS[0], at_the_fire()[NEEDS[0] as usize]);
        for kind in NEEDS {
            assert_eq!(
                needs.shortfall(kind, at_the_fire()[kind as usize]),
                first,
                "every need at its own low mark is equally short"
            );
        }
    }

    #[test]
    fn every_need_recovers_when_met_and_slips_when_it_is_not() {
        for kind in NEEDS {
            let half = need_at(NEED_MAX / 2.0);
            // Recreation is the exception and deliberately so: it is filled by
            // a thing happening rather than by a citizen being somewhere, so
            // there is nothing for the ordinary recovery term to do.
            if kind != NeedKind::Recreation {
                assert!(
                    need_step(half, kind, true, 1.0, at_the_fire()[kind as usize]).level
                        > half.level,
                    "{kind:?} must recover while it is being met"
                );
            } else {
                let mut needs = Needs::newcomer();
                set(&mut needs, kind, NEED_MAX / 2.0, false);
                needs.amuse(10.0);
                assert!(
                    needs.level(kind) > NEED_MAX / 2.0,
                    "{kind:?} must be liftable by attending something"
                );
            }
            assert!(
                need_step(half, kind, false, 1.0, at_the_fire()[kind as usize]).level < half.level,
                "{kind:?} must slip while it is neglected"
            );
        }
    }

    #[test]
    fn no_need_runs_past_full_or_below_empty() {
        for kind in NEEDS {
            assert_eq!(
                need_step(
                    need_at(NEED_MAX),
                    kind,
                    true,
                    1.0,
                    at_the_fire()[kind as usize]
                )
                .level,
                NEED_MAX
            );
            assert_eq!(
                need_step(need_at(0.0), kind, false, 1.0, at_the_fire()[kind as usize]).level,
                0.0
            );
        }
    }

    #[test]
    fn shelter_slows_a_neglected_need_without_reversing_it() {
        let start = need_at(NEED_MAX / 2.0);
        let exposed = need_step(
            start,
            NeedKind::Warmth,
            false,
            1.0,
            at_the_fire()[NeedKind::Warmth as usize],
        );
        let sheltered = need_step(
            start,
            NeedKind::Warmth,
            false,
            SHELTER_DRAIN_FACTOR,
            at_the_fire()[NeedKind::Warmth as usize],
        );
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
            need_step(
                start,
                NeedKind::Warmth,
                true,
                SHELTER_DRAIN_FACTOR,
                at_the_fire()[NeedKind::Warmth as usize]
            )
            .level,
            need_step(
                start,
                NeedKind::Warmth,
                true,
                1.0,
                at_the_fire()[NeedKind::Warmth as usize]
            )
            .level
        );
    }

    #[test]
    fn every_need_presses_only_at_its_own_thresholds() {
        for kind in NEEDS {
            let rules = kind.rules();
            let mid = (rules.low + rules.high) / 2.0;
            let calm = need_step(need_at(mid), kind, false, 1.0, at_the_fire()[kind as usize]);
            assert!(
                !calm.pressing || calm.level <= rules.low,
                "{kind:?} must not start pressing while still inside its band"
            );
            let latched = Need {
                level: mid,
                pressing: true,
                burden: 0.0,
            };
            let tended = need_step(latched, kind, true, 1.0, at_the_fire()[kind as usize]);
            assert!(
                tended.pressing || tended.level >= rules.high,
                "{kind:?} must not stop pressing while still inside its band"
            );
            assert!(
                need_step(
                    need_at(rules.low),
                    kind,
                    false,
                    1.0,
                    at_the_fire()[kind as usize]
                )
                .pressing
            );
            assert!(
                !need_step(
                    Need {
                        level: rules.high,
                        pressing: true,
                        burden: 0.0
                    },
                    kind,
                    true,
                    1.0,
                    at_the_fire()[kind as usize]
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
            assert!(rules.decay > 0.0, "{kind:?} never falls, so it is inert");
            assert!(
                rules.recovery > 0.0 || kind == NeedKind::Recreation,
                "{kind:?} has no way back up"
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
            let next = need_step(
                need,
                NeedKind::Rest,
                need.pressing,
                1.0,
                at_the_fire()[NeedKind::Rest as usize],
            );
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
        let order = needs.pressing_by_urgency(&at_the_fire());
        assert_eq!(order.len(), NEED_COUNT);
        assert_eq!(order[0], NeedKind::Food, "the emptiest need leads");
    }

    #[test]
    fn a_calm_citizen_presses_for_nothing() {
        let needs = Needs::newcomer();
        assert!(
            needs.pressing_by_urgency(&at_the_fire()).is_empty(),
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
            needs.pressing_by_urgency(&at_the_fire()),
            vec![NeedKind::Rest, NeedKind::Food]
        );
    }

    #[test]
    fn a_fatal_need_outranks_a_load_but_tiredness_does_not() {
        let mut needs = Needs::newcomer();
        set(&mut needs, NeedKind::Rest, NeedKind::Rest.rules().low, true);
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Wood), true, &at_the_fire(), None),
            Duty::Deliver,
            "wood is dropped off before bed"
        );
        assert_eq!(
            choose_duty(&needs, None, true, &at_the_fire(), None),
            Duty::Rest
        );

        set(&mut needs, NeedKind::Food, 1.0, true);
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Wood), true, &at_the_fire(), None),
            Duty::Eat,
            "a starving citizen puts the load down"
        );
    }

    #[test]
    fn a_citizen_with_nothing_pressing_goes_to_work() {
        let needs = Needs::newcomer();
        assert_eq!(
            choose_duty(&needs, None, true, &at_the_fire(), None),
            Duty::Gather
        );
        assert_eq!(
            choose_duty(&needs, Some(Cargo::Food), true, &at_the_fire(), None),
            Duty::Deliver
        );
    }

    #[test]
    fn each_duty_walks_to_its_own_destination() {
        let home = IVec2::new(3, 4);
        let drop_off = IVec2::new(7, 8);
        let patch = IVec2::new(1, 1);
        let lit = air(FULL_BURN_FUEL, 0);
        assert_eq!(
            duty_target(
                Duty::WarmUp,
                Trade::Laborer,
                &lit,
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: Some(patch),
                    stage: None,
                },
            ),
            CENTER
        );
        assert_eq!(
            duty_target(
                Duty::Eat,
                Trade::Laborer,
                &lit,
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: Some(patch),
                    stage: None,
                },
            ),
            CENTER
        );
        assert_eq!(
            duty_target(
                Duty::Deliver,
                Trade::Laborer,
                &lit,
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: Some(patch),
                    stage: None,
                },
            ),
            drop_off
        );
        assert_eq!(
            duty_target(
                Duty::Rest,
                Trade::Laborer,
                &lit,
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: Some(patch),
                    stage: None,
                },
            ),
            home
        );
        assert_eq!(
            duty_target(
                Duty::Gather,
                Trade::Laborer,
                &lit,
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: Some(patch),
                    stage: None,
                },
            ),
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
                Trade::Laborer,
                &air(FULL_BURN_FUEL, 0),
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: None,
                    stage: None,
                },
            ),
            CENTER,
            "while the fire burns, idle citizens huddle around it"
        );
        assert_eq!(
            duty_target(
                Duty::Gather,
                Trade::Laborer,
                &air(0, 0),
                CENTER,
                home,
                Errands {
                    drop_off,
                    source: None,
                    stage: None,
                },
            ),
            home,
            "once it is out, the only shelter left is their own roof"
        );
    }

    #[test]
    fn the_evening_is_over_once_it_has_happened() {
        let hour = PERFORMANCE_HOUR * TICKS_PER_HOUR;
        assert_eq!(ticks_until_performance(hour - 1), 1);
        assert_eq!(
            ticks_until_performance(hour),
            ticks_per_day(),
            "the hour it starts, the next one anybody can walk to is tomorrow"
        );
        assert_eq!(ticks_until_performance(hour + 1), ticks_per_day() - 1);
    }

    #[test]
    fn nobody_leaves_the_work_early_for_the_evening() {
        let stage = CENTER;
        let at = CENTER + IVec2::new(6, 0);
        let hour = PERFORMANCE_HOUR * TICKS_PER_HOUR;
        assert_eq!(
            stage_to_attend(&[stage], at, hour - 8),
            None,
            "eight hours out, a six-cell walk is two hours of work still to do"
        );
        assert_eq!(
            stage_to_attend(&[stage], at, hour - 6),
            Some(stage),
            "the walk starts when it is exactly as long as the time left"
        );
    }

    #[test]
    fn an_evening_calls_only_as_far_as_a_citizen_can_walk() {
        let stage = CENTER;
        let too_far = CENTER + IVec2::new(PERFORMANCE_CALL as i32 + 1, 0);
        for tick in 0..ticks_per_day() {
            assert_eq!(
                stage_to_attend(&[stage], too_far, tick),
                None,
                "nobody sets out on a walk longer than the call"
            );
        }
        let reachable = CENTER + IVec2::new(PERFORMANCE_CALL as i32, 0);
        assert_eq!(
            stage_to_attend(&[stage], reachable, PERFORMANCE_HOUR - PERFORMANCE_CALL),
            Some(stage),
            "and the furthest one who can make it sets out at the call"
        );
    }

    #[test]
    fn the_nearest_stage_is_the_one_worth_walking_to() {
        let near = CENTER + IVec2::new(2, 0);
        let far = CENTER + IVec2::new(5, 0);
        assert_eq!(
            stage_to_attend(&[far, near], CENTER, PERFORMANCE_HOUR - 2),
            Some(near)
        );
    }

    #[test]
    fn a_survival_need_outranks_the_evening() {
        let stage = Some(CENTER);
        let mut bored = contented();
        set(&mut bored, NeedKind::Recreation, 0.0, true);
        assert_eq!(
            choose_duty(&bored, None, true, &at_the_fire(), stage),
            Duty::Attend,
            "with nothing else the matter, a bored citizen goes"
        );
        assert_eq!(
            choose_duty(&bored, None, true, &at_the_fire(), None),
            Duty::Gather,
            "and works when nobody is offering"
        );
        assert_eq!(
            choose_duty(&bored, Some(Cargo::Wood), true, &at_the_fire(), stage),
            Duty::Deliver,
            "a load the colony is waiting on comes first"
        );
        let mut starving = bored;
        set(&mut starving, NeedKind::Food, 0.0, true);
        assert_eq!(
            choose_duty(&starving, None, true, &at_the_fire(), stage),
            Duty::Eat,
            "and so does anything that kills"
        );
    }

    #[test]
    fn a_child_stays_in_whatever_is_on_that_evening() {
        let mut bored = contented();
        set(&mut bored, NeedKind::Recreation, 0.0, true);
        assert_eq!(
            choose_duty(&bored, None, false, &at_the_fire(), Some(CENTER)),
            Duty::Rest,
            "children are mouths, not an audience"
        );
    }

    #[test]
    fn a_citizen_called_to_the_evening_walks_to_the_stage() {
        let stage = CENTER + IVec2::new(3, 3);
        let home = IVec2::new(-5, -5);
        let lit = air(FULL_BURN_FUEL, 0);
        assert_eq!(
            duty_target(
                Duty::Attend,
                Trade::Woodcutter,
                &lit,
                home,
                home,
                Errands {
                    drop_off: home,
                    source: Some(IVec2::new(9, 9)),
                    stage: Some(stage),
                },
            ),
            stage
        );
    }

    #[test]
    fn an_entertainer_spends_the_day_where_the_evening_will_be() {
        let home = IVec2::new(-5, -5);
        let patch = IVec2::new(9, 9);
        let lit = air(FULL_BURN_FUEL, 0);
        assert_eq!(
            duty_target(
                Duty::Gather,
                Trade::Laborer,
                &lit,
                home,
                home,
                Errands {
                    drop_off: home,
                    source: Some(patch),
                    stage: None,
                },
            ),
            patch,
            "every other trade walks to the work"
        );
        assert_eq!(
            duty_target(
                Duty::Gather,
                Trade::Entertainer,
                &lit,
                home,
                home,
                Errands {
                    drop_off: home,
                    source: Some(patch),
                    stage: None,
                },
            ),
            lit.warmth_target(home, home),
            "an entertainer holds the warm ground instead, with work in reach"
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
        let mut patches = placed(vec![
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
            gather_source(&mut patches, Cargo::Wood, from, true),
            Some((CENTER + IVec2::new(0, 4), Cargo::Food)),
            "a stripped forest sends the hauler after game instead"
        );
        patches.take(CENTER + IVec2::new(0, 4), 5);
        assert_eq!(gather_source(&mut patches, Cargo::Wood, from, true), None);
    }

    #[test]
    fn a_hauler_prefers_its_own_kind_when_both_are_standing() {
        let mut patches = placed(vec![
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
            gather_source(&mut patches, Cargo::Wood, CENTER, true),
            Some((CENTER + IVec2::new(9, 0), Cargo::Wood)),
            "the nearer patch of the wrong kind does not win"
        );
    }

    #[test]
    fn taking_from_a_patch_draws_down_that_one_cell() {
        let pos = CENTER + IVec2::new(4, 0);
        let mut patches = placed(vec![Patch {
            pos,
            kind: Cargo::Wood,
            amount: 2,
            cap: 2,
        }]);
        assert_eq!(patches.take(pos, 1), 1);
        assert_eq!(standing(&mut patches, pos), 1);
        assert_eq!(
            patches.take(pos, 5),
            1,
            "a patch hands over only what is standing on it"
        );
        assert_eq!(
            standing(&mut patches, pos),
            0,
            "a stripped patch must not underflow"
        );
        assert_eq!(patches.take(pos, 3), 0);
        assert_eq!(
            patches.take(CENTER, 1),
            0,
            "and there is nothing to lift where there is no patch"
        );
    }

    #[test]
    fn food_goes_to_the_granary_even_while_a_house_is_going_up() {
        let site = IVec2::new(1, 2);
        assert_eq!(delivery_target(Cargo::Wood, true, Some(site), None), site);
        assert_eq!(
            delivery_target(Cargo::Wood, false, Some(site), None),
            CENTER
        );
        assert_eq!(
            delivery_target(Cargo::Food, true, Some(site), None),
            CENTER,
            "nobody builds a house out of venison"
        );
    }

    #[test]
    fn wood_goes_to_the_fire_when_nothing_is_being_built() {
        assert_eq!(delivery_target(Cargo::Wood, true, None, None), CENTER);
        assert_eq!(delivery_target(Cargo::Wood, false, None, None), CENTER);
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
    fn growth_needs_the_needs_it_runs_on_met_and_both_stores_at_target() {
        let full = [1.0; NEED_COUNT];
        let pop = 30;
        let fuel = (pop as f32 * FUEL_PER_CITIZEN).ceil() as u32;
        let food = (pop as f32 * FOOD_PER_CITIZEN).ceil() as u32;
        assert!(colony_thrives(full, fuel, food, pop));
        assert!(!colony_thrives(full, 0, food, pop), "no wood");
        assert!(!colony_thrives(full, fuel, 0, pop), "no food");
        for kind in NEEDS.into_iter().filter(|kind| kind.is_survival()) {
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

    fn mayor_leaning(building: Building, weight: f32) -> Mayor {
        let mut bias = [0.0; BUILDING_COUNT];
        bias[building as usize] = weight;
        Mayor {
            bias,
            ..Mayor::default()
        }
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
        assert!(upgraded.fires[0].output > plain.fires[0].output);
        let reach = |lit: &Air| {
            (0..=R)
                .filter(|d| lit.heat_at(CENTER + IVec2::new(*d, 0)) > 0.0)
                .count()
        };
        assert!(
            reach(&upgraded) > reach(&plain),
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
        let patches: Vec<IVec2> = founding_world()
            .into_iter()
            .map(|patch| patch.pos)
            .collect();
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
        let patches = founding_world();
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
            desperate.shortfall(NeedKind::Warmth, at_the_fire()[NeedKind::Warmth as usize])
                > desperate.shortfall(NeedKind::Food, at_the_fire()[NeedKind::Food as usize]),
            "this citizen is worse off for cold than for hunger"
        );
        assert_eq!(
            choose_duty(&desperate, None, true, &at_the_fire(), None),
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
        let spans: Vec<f32> = (0..500u64)
            .map(|seed| lifespan_of(seed, FORMATION_NEUTRAL, 1.0))
            .collect();
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
        assert_eq!(
            choose_duty(&calm, None, true, &at_the_fire(), None),
            Duty::Gather
        );
        assert_eq!(
            choose_duty(&calm, None, false, &at_the_fire(), None),
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
        assert_eq!(
            choose_duty(&hungry, None, false, &at_the_fire(), None),
            Duty::Eat
        );
        let mut cold = Needs::newcomer();
        set(
            &mut cold,
            NeedKind::Warmth,
            NeedKind::Warmth.rules().low,
            true,
        );
        assert_eq!(
            choose_duty(&cold, None, false, &at_the_fire(), None),
            Duty::WarmUp
        );
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
        let midwinter = one_fire(
            generator_output(FULL_BURN_FUEL, 0),
            ambient_at(day_of(year, DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2)),
        );
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
            let air = one_fire(generator_output(FULL_BURN_FUEL, upgrades), ambient);
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
        let mut patches = placed(vec![
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
        assert!(gather_source(&mut patches, Cargo::Wood, CENTER, true).is_some());
        assert_eq!(
            gather_source(&mut patches, Cargo::Wood, CENTER, false),
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
            need = need_step(
                need,
                kind,
                need.level <= kind.rules().low || tick > 60,
                1.0,
                at_the_fire()[kind as usize],
            );
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
            need = need_step(need, kind, false, 1.0, at_the_fire()[kind as usize]);
        }
        assert!(need.burden > 0.0, "a hunger nobody answers is what costs");
        let owed = need.burden;
        for _ in 0..500 {
            need = need_step(need, kind, true, 1.0, at_the_fire()[kind as usize]);
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
            needs.spend(true, CENTER);
        }
        assert!(needs.detour_burden() > needs.get(NeedKind::Food).burden);
        assert_eq!(
            vote_of(&needs),
            building_for(NeedKind::Food),
            "the two tiers are never weighed against each other"
        );
    }

    #[test]
    fn a_citizen_the_colony_is_holding_up_votes_on_what_wastes_its_day() {
        let mut needs = Needs::newcomer();
        for _ in 0..100 {
            needs.spend(true, CENTER);
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
        assert_eq!(vote_of(&needs), building_for(NeedKind::Rest));
    }

    #[test]
    fn the_ballot_forgets_both_the_needs_and_the_hours() {
        let mut needs = owed(NeedKind::Food, 500.0);
        for _ in 0..100 {
            needs.spend(true, CENTER);
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
            median(&mut [], FORMATION_NEUTRAL),
            FORMATION_NEUTRAL,
            "an empty colony is unremarkable"
        );
        assert_eq!(median(&mut [0.4], FORMATION_NEUTRAL), 0.4);
        assert_eq!(
            median(&mut [0.9, 0.1, 0.5], FORMATION_NEUTRAL),
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

    #[test]
    fn a_body_hardens_in_the_cold_and_softens_out_of_it() {
        let mut acclimated = 0.0;
        for _ in 0..(ticks_per_day() * DAYS_PER_SEASON) {
            acclimated = acclimation_step(acclimated, true);
        }
        let hardened = acclimated;
        assert!(hardened > 0.5, "a season of working the cold should tell");
        for _ in 0..(ticks_per_day() * DAYS_PER_SEASON) {
            acclimated = acclimation_step(acclimated, false);
        }
        assert!(
            acclimated < hardened,
            "and it goes again once the exposure stops"
        );
    }

    #[test]
    fn acclimation_never_leaves_its_range() {
        let mut acclimated = 0.0;
        for _ in 0..(ticks_per_year() * 5) {
            acclimated = acclimation_step(acclimated, true);
        }
        assert!(acclimated <= 1.0);
        for _ in 0..(ticks_per_year() * 5) {
            acclimated = acclimation_step(acclimated, false);
        }
        assert!(acclimated >= 0.0);
    }

    #[test]
    fn a_better_raised_citizen_stands_a_cold_night_longer() {
        let age = FRAILTY_ONSET + 5.0;
        let frail = cold_resistance(age, LIFESPAN_BASE, STAT_MIN, 0.0);
        let sturdy = cold_resistance(age, LIFESPAN_BASE, STAT_MAX, 0.0);
        assert!(
            sturdy > frail,
            "what the colony raised has to count for something"
        );
    }

    #[test]
    fn a_winter_of_working_outside_counts_for_something_too() {
        let age = FRAILTY_ONSET + 5.0;
        let soft = cold_resistance(age, LIFESPAN_BASE, FORMATION_NEUTRAL, 0.0);
        let seasoned = cold_resistance(age, LIFESPAN_BASE, FORMATION_NEUTRAL, 1.0);
        assert!(seasoned > soft);
        assert!(
            seasoned - soft <= ACCLIMATION_WORTH + 1e-6,
            "but not more than a body can learn"
        );
    }

    #[test]
    fn nothing_a_citizen_learned_outlasts_their_own_span() {
        let done = cold_resistance(LIFESPAN_BASE * 2.0, LIFESPAN_BASE, STAT_MAX, 1.0);
        assert!(
            done <= ACCLIMATION_WORTH + 1e-6,
            "past their span all that is left is what the winters taught them"
        );
    }

    #[test]
    fn resistance_stays_a_probability() {
        for raised in [STAT_MIN, FORMATION_NEUTRAL, STAT_MAX] {
            for acclimated in [0.0, 0.5, 1.0] {
                let resistance = cold_resistance(0.0, LIFESPAN_BASE, raised, acclimated);
                assert!((0.0..=1.0).contains(&resistance));
            }
        }
    }

    #[test]
    fn a_comfortable_colony_cannot_account_for_its_own_people() {
        assert!(residual_share(1.0) > residual_share(0.0));
        assert!(
            residual_share(0.0) > 0.0,
            "even a starving colony does not explain everybody"
        );
        for prosperity in [-1.0, 0.0, 0.5, 1.0, 2.0] {
            let share = residual_share(prosperity);
            assert!(
                (0.0..=1.0).contains(&share),
                "share left its range at {prosperity}"
            );
        }
    }

    #[test]
    fn a_better_raised_citizen_outlives_a_worse_raised_one() {
        let lean = 0.0;
        assert!(
            lifespan_of(9, STAT_MAX, lean) > lifespan_of(9, STAT_MIN, lean),
            "in a colony that can account for its people, upbringing decides"
        );
    }

    #[test]
    fn a_starving_colony_can_read_its_people_off_the_granary() {
        let told_by_upbringing = |prosperity: f32| {
            lifespan_of(9, STAT_MAX, prosperity) - lifespan_of(9, STAT_MIN, prosperity)
        };
        assert!(
            told_by_upbringing(0.0) > told_by_upbringing(1.0),
            "and a comfortable one produces people who differ for reasons it cannot see"
        );
    }

    #[test]
    fn two_citizens_raised_alike_still_do_not_die_together() {
        assert_ne!(
            lifespan_of(21, FORMATION_NEUTRAL, 1.0),
            lifespan_of(22, FORMATION_NEUTRAL, 1.0)
        );
    }

    #[test]
    fn a_span_stays_a_span() {
        for seed in 0..200u64 {
            for raised in [STAT_MIN, FORMATION_NEUTRAL, STAT_MAX] {
                for prosperity in [0.0, 0.5, 1.0] {
                    let span = lifespan_of(seed, raised, prosperity);
                    assert!(
                        span > FRAILTY_ONSET,
                        "nobody is born already past frailty: {span}"
                    );
                    assert!(span < LIFESPAN_BASE * 2.0);
                }
            }
        }
    }

    #[test]
    fn a_childhood_remembers_how_good_the_whole_of_it_was() {
        let mut childhood = Upbringing::born(4);
        for _ in 0..100 {
            childhood.observe(1.0, 1.0);
        }
        for _ in 0..100 {
            childhood.observe(0.0, 0.0);
        }
        let prosperity = childhood.prosperity();
        assert!(
            (prosperity - 0.5).abs() < 0.01,
            "half a childhood of plenty and half of want averages out, not {prosperity}"
        );
    }

    #[test]
    fn a_childhood_nobody_watched_reads_as_neither_good_nor_bad() {
        assert_eq!(Upbringing::grown(4).prosperity(), FORMATION_NEUTRAL);
    }

    #[test]
    fn settling_says_when_a_child_has_finished_becoming_someone() {
        let mut childhood = Upbringing::born(6);
        for age in MILESTONE_AGES {
            let grown = childhood.settle_due(age);
            assert_eq!(
                grown,
                age == ADULT_AGE,
                "only the last milestone finishes a childhood"
            );
        }
    }

    #[test]
    fn every_trade_leans_on_a_stat_and_most_of_them_fetch_something() {
        let mut fetched = Vec::new();
        for trade in TRADES {
            let _ = trade_stat(trade);
            if let Some(cargo) = trade_cargo(trade) {
                assert!(!fetched.contains(&cargo), "{trade:?} duplicates a trade");
                fetched.push(cargo);
            }
        }
        assert_eq!(
            fetched.len(),
            CARGO_COUNT,
            "every store wants somebody fetching it"
        );
        assert_eq!(
            trade_cargo(Trade::Laborer),
            None,
            "a labourer goes wherever the colony is shorter"
        );
    }

    #[test]
    fn practice_and_the_body_both_count_towards_fitting_the_work() {
        let raw = trade_fit(FORMATION_NEUTRAL, 0.0);
        assert!(trade_fit(STAT_MAX, 0.0) > raw, "what a citizen is counts");
        assert!(
            trade_fit(FORMATION_NEUTRAL, 1.0) > raw,
            "so does what they have done"
        );
        assert!(trade_fit(STAT_MIN, 0.0) > 0.0, "nobody is worth nothing");
    }

    #[test]
    fn fit_does_not_run_away_with_itself() {
        assert!(trade_fit(STAT_MAX, 1.0) <= FIT_CEILING);
        assert!(
            trade_fit(STAT_MAX, 10.0) <= FIT_CEILING,
            "practice saturates"
        );
    }

    #[test]
    fn the_focus_layer_is_wired_but_empty() {
        let base = 0.6;
        assert_eq!(
            effective_stat(base, 1.0, 1.0),
            base,
            "until the mood layer lands, focus must change nothing"
        );
        assert!(effective_stat(base, 1.0, 1.5) > base);
    }

    /// What a citizen averages over many trips, which is what the colony feels.
    fn carried_per_trip(effective: f32) -> f32 {
        let trips = 2000;
        let mut banked = 0.0;
        let mut total = 0u32;
        for _ in 0..trips {
            let (load, left) = haul_load(effective, banked);
            total += load;
            banked = left;
        }
        total as f32 / trips as f32
    }

    #[test]
    fn what_a_citizen_carries_home_rises_with_no_step_in_it() {
        let mut previous = 0.0;
        for notch in 0..=36 {
            let effective = notch as f32 * 0.05;
            let carried = carried_per_trip(effective);
            assert!(
                carried > previous,
                "a step at effective {effective}: {carried} is no better than {previous}"
            );
            previous = carried;
        }
    }

    #[test]
    fn a_hauler_who_is_owed_a_fraction_is_paid_it_eventually() {
        let effective = 0.42;
        let expected = HAUL_BASE + effective * HAUL_LOAD_SWING;
        let carried = carried_per_trip(effective);
        assert!(
            (carried - expected).abs() < 0.01,
            "the banked fraction has to come back: {carried} against {expected}"
        );
    }

    #[test]
    fn a_trip_never_lifts_nothing() {
        // The patch is stripped for a load at the moment it is picked up, so a
        // trip that carried nothing would have destroyed what it left behind.
        for effective in [-5.0, 0.0, 0.01, FIT_CEILING] {
            let (load, banked) = haul_load(effective, 0.0);
            assert!(
                load >= 1,
                "a trip that lifts nothing strips the treeline for free"
            );
            assert!((0.0..1.0).contains(&banked));
        }
    }

    #[test]
    fn a_nearer_more_practised_citizen_scores_better() {
        let far = assignment_score(20, 0.0, 0.0);
        let near = assignment_score(2, 0.0, 0.0);
        assert!(
            near > far,
            "distance is the wrong distance, but it is still a cost"
        );
        assert!(assignment_score(2, 1.0, 0.0) > near, "practice counts");
        assert!(assignment_score(2, 0.0, 1.0) > near, "and the office leans");
    }

    #[test]
    fn a_vacancy_goes_to_one_of_the_best_few_rather_than_the_best() {
        let scored = vec![(0, 9.0), (1, 8.0), (2, 7.0), (3, 1.0)];
        let mut taken = Vec::new();
        for roll in 0..20u64 {
            let pick = pick_from_top(&scored, noise(roll, 7)).expect("somebody is available");
            assert_ne!(pick, 3, "the hopeless candidate is never in the running");
            if !taken.contains(&pick) {
                taken.push(pick);
            }
        }
        assert!(
            taken.len() > 1,
            "always the same pick is argmax with extra steps"
        );
    }

    #[test]
    fn an_empty_field_leaves_the_vacancy_open() {
        assert_eq!(pick_from_top(&[], 0.5), None);
    }

    #[test]
    fn practice_is_gained_at_the_work_and_rusts_away_from_it() {
        let mut experience = 0.0;
        for _ in 0..(ticks_per_season() * 2) {
            experience = experience_step(experience, true);
        }
        let practised = experience;
        assert!(practised > 0.0);
        assert!(practised <= 1.0, "practice tops out");
        for _ in 0..(ticks_per_season() * 4) {
            experience = experience_step(experience, false);
        }
        assert!(experience < practised, "and goes again if the work stops");
        assert!(experience >= 0.0);
    }

    #[test]
    fn the_colony_holds_back_hands_for_work_no_trade_covers() {
        for hands in [10usize, 30, 60] {
            let held = laborers_wanted(hands);
            let share = held as f32 / hands as f32;
            assert!(
                (0.20..=0.25).contains(&share),
                "{held} of {hands} is {share}, outside the fifth-to-quarter band"
            );
        }
        assert_eq!(laborers_wanted(0), 0);
    }

    #[test]
    fn a_child_takes_the_house_trade_with_some_of_the_practice_in_it() {
        let head = 0.8;
        let child = inherited_experience(head);
        assert!(
            child > 0.0,
            "a child of the house starts ahead of a stranger"
        );
        assert!(child < head, "but not level with the one who did the work");
    }

    #[test]
    fn a_child_of_the_house_takes_the_head_of_it_s_trade() {
        let home = IVec2::new(4, 4);
        let elsewhere = IVec2::new(9, 9);
        let adults = [
            (home, Trade::Hunter, 0.9, 40.0),
            (home, Trade::Woodcutter, 0.5, 25.0),
            (elsewhere, Trade::Woodcutter, 1.0, 60.0),
        ];
        let (trade, practice) = household_head(&adults, home).expect("the house has a head");
        assert_eq!(
            trade,
            Trade::Hunter,
            "the oldest of the house, not of the colony"
        );
        assert_eq!(practice, 0.9);
        assert_eq!(
            household_head(&adults, IVec2::new(1, 1)),
            None,
            "a house with no grown citizen in it hands down nothing"
        );
    }

    #[test]
    fn the_founding_party_sets_aside_its_labourers_first() {
        let mut counts = [0usize; TRADE_COUNT];
        for index in 0..CITIZENS {
            counts[founding_trade(index) as usize] += 1;
        }
        assert_eq!(counts[Trade::Laborer as usize], laborers_wanted(CITIZENS));
        for trade in [Trade::Woodcutter, Trade::Hunter] {
            assert!(
                counts[trade as usize] > 0,
                "somebody has to fetch {trade:?}"
            );
        }
        assert_eq!(
            counts[Trade::Entertainer as usize],
            0,
            "the founding party is too lean to carry one"
        );
    }

    #[test]
    fn a_word_is_measured_against_the_colony_and_not_a_constant() {
        let middle = 0.5;
        assert_eq!(regard_of(middle, middle), Regard::Middling);
        assert_eq!(
            regard_of(middle, 0.2),
            Regard::Strong,
            "strong in a weak colony"
        );
        assert_eq!(
            regard_of(middle, 0.8),
            Regard::Poor,
            "and poor in a strong one"
        );
    }

    #[test]
    fn the_bands_run_in_order() {
        let middle = 0.5;
        let ladder = [
            regard_of(0.0, middle),
            regard_of(middle - REGARD_STEP, middle),
            regard_of(middle, middle),
            regard_of(middle + REGARD_STEP, middle),
            regard_of(1.0, middle),
        ];
        for pair in ladder.windows(2) {
            assert!(
                pair[0] as usize <= pair[1] as usize,
                "the ladder is out of order"
            );
        }
        assert_eq!(ladder[0], Regard::Poor);
        assert_eq!(ladder[4], Regard::Strong);
    }

    #[test]
    fn every_band_has_a_word_and_no_two_share_one() {
        let mut said = Vec::new();
        for band in [
            Regard::Poor,
            Regard::Below,
            Regard::Middling,
            Regard::Above,
            Regard::Strong,
        ] {
            let word = band.word();
            assert!(!word.is_empty());
            assert!(
                !said.contains(&word),
                "{band:?} says what another band says"
            );
            said.push(word);
        }
    }

    #[test]
    fn the_colony_guesses_from_the_childhood_it_watched_before_it_has_watched_the_work() {
        let truth = STAT_MAX;
        let fat = estimate(truth, 1.0, 0.0);
        let lean = estimate(truth, 0.0, 0.0);
        assert!(
            fat > lean,
            "a child of a fat decade starts described higher"
        );
        assert_ne!(fat, truth, "but the colony is not claiming to know");
    }

    #[test]
    fn the_estimate_converges_on_what_a_citizen_actually_is() {
        let truth = STAT_MAX;
        let green = estimate(truth, 0.0, 0.0);
        let half = estimate(truth, 0.0, WORKER_DAYS_TO_KNOW / 2.0);
        let known = estimate(truth, 0.0, WORKER_DAYS_TO_KNOW);
        assert!(
            (green - truth).abs() > (half - truth).abs(),
            "watching has to move the estimate"
        );
        assert!((half - truth).abs() > (known - truth).abs());
        assert_eq!(known, truth, "and land on it");
        assert_eq!(
            estimate(truth, 0.0, WORKER_DAYS_TO_KNOW * 10.0),
            truth,
            "and stay there"
        );
    }

    #[test]
    fn a_prior_starts_a_citizen_in_a_band_and_never_at_a_value() {
        for prosperity in [0.0, 0.5, 1.0] {
            let guessed = estimate(STAT_MAX, prosperity, 0.0);
            assert!(
                (guessed - STAT_MAX).abs() > 1e-3,
                "a childhood is a band, not a measurement"
            );
            assert!((0.0..=1.0).contains(&guessed));
        }
    }

    #[test]
    fn the_colony_knows_a_citizen_once_it_has_watched_them_work() {
        assert!(!is_known(0.0));
        assert!(!is_known(WORKER_DAYS_TO_KNOW - 1.0));
        assert!(is_known(WORKER_DAYS_TO_KNOW));
    }

    #[test]
    fn a_posting_goes_stale_only_after_it_has_stood_a_while() {
        assert!(!posting_is_stale(0.0));
        assert!(!posting_is_stale(POSTING_STALE_AFTER - 1.0));
        assert!(posting_is_stale(POSTING_STALE_AFTER));
    }

    #[test]
    fn a_posting_ages_while_the_store_is_short_and_is_torn_down_when_it_is_not() {
        let mut postings = Postings::default();
        for _ in 0..10 {
            postings.note(Cargo::Wood, true);
        }
        assert_eq!(postings.days_short(Cargo::Wood), 10.0);
        assert_eq!(
            postings.days_short(Cargo::Food),
            0.0,
            "one store going short says nothing about the other"
        );
        postings.note(Cargo::Wood, false);
        assert_eq!(
            postings.days_short(Cargo::Wood),
            0.0,
            "a store that is filled again has no posting standing"
        );
    }

    #[test]
    fn a_fresh_posting_is_the_labourers_and_a_stale_one_is_anybodys() {
        let wanted = Trade::Woodcutter;
        assert!(
            may_be_taken(Trade::Laborer, wanted, false),
            "spare hands always"
        );
        assert!(
            !may_be_taken(Trade::Hunter, wanted, false),
            "while the posting is fresh, nobody leaves a trade for it"
        );
        assert!(
            may_be_taken(Trade::Hunter, wanted, true),
            "a job nobody has taken for long enough stops being somebody's job"
        );
        assert!(
            !may_be_taken(wanted, wanted, true),
            "and the people already doing it are not the answer to it"
        );
    }

    #[test]
    fn a_cell_belongs_to_one_chunk_on_either_side_of_the_hearth() {
        assert_eq!(chunk_of(IVec2::new(0, 0)), IVec2::new(0, 0));
        assert_eq!(chunk_of(IVec2::new(CHUNK - 1, CHUNK - 1)), IVec2::new(0, 0));
        assert_eq!(chunk_of(IVec2::new(CHUNK, 0)), IVec2::new(1, 0));
        assert_eq!(
            chunk_of(IVec2::new(-1, -1)),
            IVec2::new(-1, -1),
            "a world with no edge has chunks on the far side of zero"
        );
        assert_eq!(chunk_of(IVec2::new(-CHUNK, -CHUNK)), IVec2::new(-1, -1));
    }

    #[test]
    fn a_chunk_comes_out_the_same_however_it_is_asked_for() {
        let world = 1234;
        let chunk = IVec2::new(3, -2);
        let once = chunk_patches(world, chunk);
        // Ask for a scatter of other chunks in between, which is the whole
        // point: order must not matter.
        for other in [IVec2::new(0, 0), IVec2::new(-9, 4), IVec2::new(3, -1)] {
            let _ = chunk_patches(world, other);
        }
        let again = chunk_patches(world, chunk);
        assert_eq!(once.len(), again.len());
        for (a, b) in once.iter().zip(&again) {
            assert_eq!(a.pos, b.pos);
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.cap, b.cap);
        }
    }

    #[test]
    fn different_chunks_and_different_worlds_hold_different_things() {
        let here = chunk_patches(7, IVec2::new(2, 2));
        let next_door = chunk_patches(7, IVec2::new(3, 2));
        let elsewhere = chunk_patches(8, IVec2::new(2, 2));
        assert_ne!(here[0].pos, next_door[0].pos);
        assert_ne!(
            here[0].pos, elsewhere[0].pos,
            "the world seed has to matter"
        );
    }

    #[test]
    fn every_patch_a_chunk_holds_is_inside_that_chunk() {
        for cx in -2..3 {
            for cy in -2..3 {
                let chunk = IVec2::new(cx, cy);
                for patch in chunk_patches(99, chunk) {
                    assert_eq!(
                        chunk_of(patch.pos),
                        chunk,
                        "a chunk that writes outside itself forces its neighbour to exist"
                    );
                }
            }
        }
    }

    #[test]
    fn no_two_patches_of_a_chunk_stand_on_one_cell() {
        for cx in -2..3 {
            let patches = chunk_patches(5, IVec2::new(cx, 1));
            for (i, a) in patches.iter().enumerate() {
                for b in &patches[i + 1..] {
                    assert_ne!(a.pos, b.pos);
                }
            }
        }
    }

    #[test]
    fn nothing_grows_on_the_hearth_or_the_ground_the_colony_builds_on() {
        // Plots are laid out on rings measured the way the heat is, so the
        // ground they stand on is cleared the same way.
        for chunk in [IVec2::new(0, 0), IVec2::new(-1, 0), IVec2::new(0, -1)] {
            for patch in chunk_patches(31, chunk) {
                let out = patch.pos.as_vec2().distance(CENTER.as_vec2());
                assert!(
                    out > PLOT_MAX_RADIUS as f32,
                    "a patch at {out} cells is standing on a building plot"
                );
            }
        }
    }

    #[test]
    fn the_field_changes_gently_from_one_cell_to_the_next() {
        let world = 42;
        for x in -80..80 {
            let here = field_at(world, IVec2::new(x, 5));
            let next = field_at(world, IVec2::new(x + 1, 5));
            assert!((0.0..=1.0).contains(&here));
            assert!(
                (here - next).abs() < 0.5,
                "a hash has no continuity; a field must have some, at x {x}"
            );
        }
    }

    #[test]
    fn the_field_is_not_flat() {
        let world = 42;
        let sampled: Vec<f32> = (0..200)
            .map(|x| field_at(world, IVec2::new(x * 3, 0)))
            .collect();
        let high = sampled.iter().copied().fold(f32::MIN, f32::max);
        let low = sampled.iter().copied().fold(f32::MAX, f32::min);
        assert!(high - low > 0.4, "a field that never varies is a constant");
    }

    #[test]
    fn the_near_ring_is_no_richer_than_it_ever_was() {
        for distance in 0..=PATCH_RADIUS {
            assert_eq!(
                richness_at(distance),
                1.0,
                "the early game must not be retuned by this"
            );
        }
        assert!(
            richness_at(PATCH_RADIUS + 1) > 1.0,
            "and it has to rise past it"
        );
    }

    #[test]
    fn richness_rises_with_distance_and_then_stops() {
        assert!(richness_at(200) > richness_at(100));
        assert_eq!(
            richness_at(10_000),
            richness_at(RICHNESS_BEST),
            "past the best of it, further is only further"
        );
    }

    #[test]
    fn the_world_near_the_hearth_is_about_as_thick_as_the_rings_were() {
        // Density rather than a count: the rings put twelve patches inside a
        // disc of radius seventeen, and that is the thing the early game was
        // balanced against.
        // Eight treelines and four hunting grounds, on a ring of radius
        // seventeen. That is the world the early game was balanced against.
        let rings_held = 12.0;
        let rings_had = rings_held / (std::f32::consts::PI * (PATCH_RADIUS * PATCH_RADIUS) as f32);
        let world = 2026;
        let reach = 2;
        let mut patches = 0;
        for cx in -reach..=reach {
            for cy in -reach..=reach {
                patches += chunk_patches(world, IVec2::new(cx, cy)).len();
            }
        }
        let across = ((2 * reach + 1) * CHUNK) as f32;
        let density = patches as f32 / (across * across);
        assert!(
            density > rings_had * 0.5 && density < rings_had * 2.0,
            "the rings ran at {rings_had} patches a cell and the chunks run at {density}"
        );
    }

    /// A world holding only these patches: every chunk a search could reach is
    /// already there and empty, so nothing the seed would have drawn gets in
    /// the way of a test that needs to name every patch a citizen can see.
    fn placed(patches: Vec<Patch>) -> Patches {
        let mut world = Patches {
            seed: 0,
            chunks: BTreeMap::new(),
            worked: BTreeMap::new(),
            known: BTreeSet::new(),
        };
        let home = chunk_of(CENTER);
        let span = SEARCH_LIMIT / CHUNK + 2;
        for x in -span..=span {
            for y in -span..=span {
                let chunk = home + IVec2::new(x, y);
                world.chunks.insert(key(chunk), Held::new(Vec::new()));
                world.known.insert(key(chunk));
            }
        }
        for patch in patches {
            if let Some(held) = world.chunks.get_mut(&key(chunk_of(patch.pos))) {
                held.standing[patch.kind as usize] += patch.amount;
                held.patches.push(patch);
            }
        }
        world
    }

    /// Everything the colony is founded on, however the chunks are held.
    fn founding_world() -> Vec<Patch> {
        Patches::new(WORLD_SEED)
            .chunks
            .values()
            .flat_map(|held| held.patches.iter())
            .copied()
            .collect()
    }

    /// Draw the world within a reach of here and then look at it. Production
    /// asks for the nearest patch of a kind and never for a whole radius, so
    /// this is the tests' way of naming an area.
    fn drawn(world: &mut Patches, at: IVec2, radius: i32) -> impl Iterator<Item = &Patch> {
        let low = chunk_of(at - IVec2::splat(radius));
        let high = chunk_of(at + IVec2::splat(radius));
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                world.realise(IVec2::new(x, y));
            }
        }
        world.seen(at, radius)
    }

    fn standing(world: &mut Patches, cell: IVec2) -> u32 {
        drawn(world, cell, 0).next().map_or(0, |patch| patch.amount)
    }

    #[test]
    fn a_query_only_ever_looks_at_what_is_near() {
        let mut world = Patches::new(WORLD_SEED);
        let radius = 24;
        for patch in drawn(&mut world, CENTER, radius) {
            let out = (patch.pos - CENTER).abs().max_element();
            assert!(out <= radius, "a bounded query looked {out} cells away");
        }
        assert!(
            drawn(&mut world, CENTER, radius).count() > 0,
            "and it found something inside the reach"
        );
    }

    #[test]
    fn a_query_never_touches_more_chunks_than_its_reach_covers() {
        let mut world = Patches::new(WORLD_SEED);
        let radius = 24;
        let held = world.chunks.len();
        // Somewhere the colony has never been, so what the query draws is all
        // of what it draws.
        let _ = drawn(&mut world, CENTER + IVec2::splat(1000), radius).count();
        let drawn = world.chunks.len() - held;
        let could_reach = ((2 * radius / CHUNK) + 2).pow(2) as usize;
        assert!(
            drawn <= could_reach,
            "{drawn} chunks drawn for a reach that covers at most {could_reach}"
        );
    }

    #[test]
    fn a_colony_cannot_walk_away_from_a_stripped_treeline_and_come_back_to_a_full_one() {
        let mut world = Patches::new(WORLD_SEED);
        let cell = drawn(&mut world, CENTER, 64)
            .map(|patch| patch.pos)
            .next()
            .expect("the near world holds something");
        let before = standing(&mut world, cell);
        assert!(before > 0);
        assert_eq!(world.take(cell, before), before, "strip it bare");
        assert_eq!(standing(&mut world, cell), 0);

        world.forget_beyond(&[], 0);
        assert_eq!(
            standing(&mut world, cell),
            0,
            "the colony took it, so it stays taken however the chunk is asked for again"
        );
    }

    #[test]
    fn a_cell_nobody_touched_comes_back_from_the_seed() {
        let mut world = Patches::new(WORLD_SEED);
        let untouched: Vec<(IVec2, u32)> = drawn(&mut world, CENTER, 64)
            .map(|patch| (patch.pos, patch.amount))
            .collect();
        world.forget_beyond(&[], 0);
        for (cell, was) in untouched {
            assert_eq!(
                standing(&mut world, cell),
                was,
                "an untouched cell must regenerate"
            );
        }
    }

    #[test]
    fn a_patch_grows_back_towards_the_cap_it_was_generated_with() {
        let mut world = Patches::new(WORLD_SEED);
        let cell = drawn(&mut world, CENTER, 64)
            .map(|p| p.pos)
            .next()
            .expect("a patch");
        let cap = drawn(&mut world, cell, 0).next().expect("a patch").cap;
        world.take(cell, cap);
        assert_eq!(standing(&mut world, cell), 0);
        for _ in 0..1000 {
            world.regrow(true);
        }
        assert_eq!(
            standing(&mut world, cell),
            cap,
            "and stops at the cap, not past it"
        );
    }

    #[test]
    fn nothing_grows_back_in_a_chunk_nobody_has_been_to() {
        let mut world = Patches::new(WORLD_SEED);
        let _ = drawn(&mut world, CENTER, 24).count();
        let realised = world.chunks.len();
        world.regrow(true);
        assert_eq!(
            world.chunks.len(),
            realised,
            "regrowth must not be what realises the world"
        );
    }

    #[test]
    fn the_search_widens_when_the_near_ground_is_bare() {
        let mut world = Patches::new(WORLD_SEED);
        let bare: Vec<IVec2> = drawn(&mut world, CENTER, NEAR_GROUND)
            .map(|patch| patch.pos)
            .collect();
        for cell in &bare {
            let all = standing(&mut world, *cell);
            world.take(*cell, all);
        }
        let found = gather_source(&mut world, Cargo::Wood, CENTER, true)
            .expect("a bare near ring must send a citizen further, not idle them");
        let out = (found.0 - CENTER).abs().max_element();
        assert!(out > NEAR_GROUND, "it found something at {out}");
        assert!(out <= SEARCH_LIMIT, "but not past where anybody would walk");
    }

    #[test]
    fn nobody_walks_past_where_the_ground_stops_getting_better() {
        // The limit is where richness stops rising rather than a number of its
        // own, and the ground the colony lives off is inside what it may walk to.
        const _: () = assert!(SEARCH_LIMIT == RICHNESS_BEST);
        const _: () = assert!(NEAR_GROUND < SEARCH_LIMIT);
    }

    #[cfg(not(feature = "window"))]
    #[test]
    fn the_bound_counts_the_cuts_the_colony_remembers_and_not_only_its_chunks() {
        let mut world = Patches::new(WORLD_SEED);
        let cell = drawn(&mut world, CENTER, 64)
            .map(|patch| patch.pos)
            .next()
            .expect("the near world holds something");
        let held = world.held_cells();
        assert_eq!(
            held,
            world.chunks.len() * (CHUNK * CHUNK) as usize + world.known.len(),
            "the chunks it holds and the ground it knows, nothing cut yet"
        );
        world.take(cell, 1);
        assert_eq!(
            world.held_cells(),
            held + 1,
            "a remembered cut is held whether or not its chunk still is"
        );
        assert!(
            world_is_bounded(WORLD_CELLS_HELD),
            "the ceiling itself must be allowed"
        );
        assert!(!world_is_bounded(WORLD_CELLS_HELD + 1), "one past it not");
    }

    #[cfg(not(feature = "window"))]
    #[test]
    fn the_world_a_wandering_colony_holds_stays_bounded() {
        let mut world = Patches::new(WORLD_SEED);
        let furthest = SEARCH_LIMIT;
        // A colony that keeps walking, cutting as it goes and never coming
        // back. Both halves of what it holds are under the ceiling: the chunks,
        // which it may drop, and the cuts, which it may not.
        for step in 1..60 {
            let out = CENTER + IVec2::splat(step * furthest);
            let cut: Vec<IVec2> = drawn(&mut world, out, furthest)
                .map(|patch| patch.pos)
                .collect();
            assert!(!cut.is_empty(), "there has to be ground out here to cut");
            for cell in cut {
                world.take(cell, 1);
            }
            world.forget_beyond(&[out], FORGET_BEYOND);
            assert!(
                world_is_bounded(world.held_cells()),
                "{} cells held after {step} moves",
                world.held_cells()
            );
        }
    }

    #[test]
    fn the_frame_holds_every_cell_the_old_disc_did() {
        for x in -R..=R {
            for y in -R..=R {
                let cell = CENTER + IVec2::new(x, y);
                if cell.as_vec2().distance(CENTER.as_vec2()) <= R as f32 {
                    assert!(on_frame(cell), "the frame lost a cell the disc held");
                }
            }
        }
    }

    #[test]
    fn the_frame_holds_the_ground_the_colony_lives_off() {
        let edge = CENTER + IVec2::new(NEAR_GROUND, NEAR_GROUND);
        assert!(
            on_frame(edge),
            "the corner of the near ground must be watchable"
        );
        assert!(
            !on_frame(edge + IVec2::splat(1)),
            "and the frame has to end somewhere, or it is a board again"
        );
    }

    #[test]
    fn a_citizen_out_past_the_frame_is_counted_rather_than_lost() {
        let out = [
            CENTER,
            CENTER + IVec2::new(VIEW_RADIUS, 0),
            CENTER + IVec2::new(VIEW_RADIUS + 1, 0),
            CENTER - IVec2::new(0, VIEW_RADIUS + 40),
        ];
        assert_eq!(out.iter().filter(|cell| !on_frame(**cell)).count(), 2);
    }

    #[test]
    fn a_hand_who_found_no_work_never_outscores_one_who_found_it_far_away() {
        let furthest = SEARCH_LIMIT;
        assert!(
            WALK_UNFOUND >= furthest,
            "not finding work has to count as at least the longest walk there is"
        );
        assert!(
            assignment_score(WALK_UNFOUND, 0.5, 0.0) <= assignment_score(furthest, 0.5, 0.0),
            "a spare hand with nowhere to go must not be the best candidate"
        );
    }

    fn mean_day() -> Air {
        one_fire(GENERATOR_HEAT, AMBIENT_MEAN)
    }

    /// The walk home counted the way a citizen actually takes it: one king move
    /// a tick, losing warmth wherever the fire does not reach.
    fn walked_home(at: IVec2, air: &Air) -> f32 {
        let mut pos = at;
        let mut cold = 0.0;
        while pos != CENTER {
            if air.heat_at(pos) < 0.0 {
                cold += 1.0;
            }
            pos = step_toward(pos, CENTER);
        }
        cold
    }

    #[test]
    fn the_walk_home_is_counted_the_way_a_citizen_walks_it() {
        for air in [
            mean_day(),
            one_fire(GENERATOR_HEAT, AMBIENT_MEAN - AMBIENT_SWING),
        ] {
            for x in -70..=70 {
                for y in -70..=70 {
                    let at = CENTER + IVec2::new(x, y);
                    assert_eq!(
                        cost_of_getting_home(at, &air),
                        walked_home(at, &air),
                        "the closed form disagrees with the walk at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_citizen_inside_the_warm_ring_has_no_walk_to_pay_for() {
        let air = mean_day();
        assert_eq!(cost_of_getting_home(CENTER, &air), 0.0);
        let rim = CENTER + IVec2::new(air.fires[0].reach(air.ambient) as i32, 0);
        assert_eq!(
            cost_of_getting_home(rim, &air),
            0.0,
            "the rim is still warm"
        );
    }

    #[test]
    fn the_old_mark_kills_a_citizen_fifty_ticks_from_the_fire() {
        let air = mean_day();
        let out = CENTER + IVec2::new(63, 0);
        let cost = cost_of_getting_home(out, &air);
        assert_eq!(cost, 50.0, "fifty ticks of cold between here and the fire");

        let fixed = NeedKind::Warmth.rules().low;
        assert_eq!(
            cost - fixed,
            25.0,
            "turning for home at the fixed mark dies twenty-five ticks short"
        );

        let looked_ahead = marks_of(NeedKind::Warmth, cost, CAUTION_BASE).low;
        assert_eq!(
            looked_ahead - cost,
            CAUTION_BASE,
            "and turning on the walk home arrives with the margin still in hand"
        );
    }

    #[test]
    fn a_citizen_at_the_fire_keeps_the_marks_they_always_had() {
        let rules = NeedKind::Warmth.rules();
        let marks = marks_of(NeedKind::Warmth, 0.0, CAUTION_BASE);
        assert_eq!(marks.low, rules.low, "nothing about the hearth changes");
        assert_eq!(marks.high, rules.high);
    }

    #[test]
    fn the_gap_between_the_marks_is_what_never_moves() {
        for kind in NEEDS {
            let rules = kind.rules();
            let band = rules.high - rules.low;
            for cost in [0.0, 7.0, 50.0, 160.0] {
                let marks = marks_of(kind, cost, CAUTION_BASE);
                assert_eq!(
                    marks.high - marks.low,
                    band,
                    "{kind:?} lost the gap hysteresis exists to protect"
                );
            }
        }
    }

    #[test]
    fn only_warmth_knows_where_it_is_standing() {
        for kind in NEEDS {
            let rules = kind.rules();
            let far = marks_of(kind, 50.0, CAUTION_BASE);
            if kind == NeedKind::Warmth {
                assert!(far.low > rules.low, "warmth has to read the walk home");
            } else {
                assert_eq!(far.low, rules.low, "{kind:?} is not answered by walking");
            }
        }
    }

    /// Every patch of a kind the colony has met within the limit, scanned one at
    /// a time. What the index has to agree with.
    fn nearest_by_scanning(world: &Patches, kind: Cargo, from: IVec2, limit: i32) -> Option<IVec2> {
        world
            .seen(from, limit)
            .filter(|patch| patch.kind == kind && patch.amount > 0)
            .map(|patch| (patch.pos, (patch.pos - from).abs().max_element()))
            .min_by_key(|(pos, walk)| (*walk, pos.x, pos.y))
            .map(|(pos, _)| pos)
    }

    #[test]
    fn the_index_finds_what_a_scan_would_have_found() {
        let mut world = Patches::new(WORLD_SEED);
        for step in 0..40 {
            let from = CENTER + IVec2::new(step * 7 - 140, 91 - step * 5);
            for kind in [Cargo::Wood, Cargo::Food] {
                let found = world.nearest(kind, from, SEARCH_LIMIT);
                // The scan has to see the same world, so it runs after the
                // index has drawn whatever it drew.
                let scanned = nearest_by_scanning(&world, kind, from, SEARCH_LIMIT);
                let walk = |p: Option<IVec2>| p.map(|c| (c - from).abs().max_element());
                assert_eq!(
                    walk(found),
                    walk(scanned),
                    "index and scan disagree about {kind:?} from {from:?}"
                );
            }
        }
    }

    #[test]
    fn nobody_looks_past_the_limit() {
        let mut world = Patches::new(WORLD_SEED);
        let from = CENTER;
        for limit in [0, 5, 24, 64, SEARCH_LIMIT] {
            if let Some(found) = world.nearest(Cargo::Wood, from, limit) {
                let walk = (found - from).abs().max_element();
                assert!(
                    walk <= limit,
                    "found work {walk} away under a limit of {limit}"
                );
            }
        }
    }

    #[test]
    fn a_chunk_with_nothing_standing_is_passed_over_unopened() {
        let mut world = Patches::new(WORLD_SEED);
        let home = chunk_of(CENTER);
        let standing: u32 = world.standing_in(home, Cargo::Wood);
        assert!(standing > 0, "the hearth's own chunk holds treelines");
        let cells: Vec<IVec2> = world
            .seen(CENTER, CHUNK)
            .filter(|patch| patch.kind == Cargo::Wood && chunk_of(patch.pos) == home)
            .map(|patch| patch.pos)
            .collect();
        for cell in cells {
            let all = drawn(&mut world, cell, 0)
                .next()
                .map_or(0, |patch| patch.amount);
            world.take(cell, all);
        }
        assert_eq!(
            world.standing_in(home, Cargo::Wood),
            0,
            "a stripped chunk has to say so, or the search will keep opening it"
        );
    }

    #[test]
    fn what_a_chunk_says_it_holds_is_what_it_holds() {
        let mut world = Patches::new(WORLD_SEED);
        let home = chunk_of(CENTER);
        let summed = |world: &Patches, kind: Cargo| -> u32 {
            world
                .seen(CENTER, CHUNK * 2)
                .filter(|patch| patch.kind == kind && chunk_of(patch.pos) == home)
                .map(|patch| patch.amount)
                .sum()
        };
        for kind in [Cargo::Wood, Cargo::Food] {
            assert_eq!(world.standing_in(home, kind), summed(&world, kind));
        }
        let cell = world
            .seen(CENTER, CHUNK)
            .find(|patch| patch.kind == Cargo::Wood && chunk_of(patch.pos) == home)
            .map(|patch| patch.pos)
            .expect("a treeline in the hearth's chunk");
        world.take(cell, 3);
        assert_eq!(
            world.standing_in(home, Cargo::Wood),
            summed(&world, Cargo::Wood),
            "taking has to keep the summary true"
        );
        for _ in 0..500 {
            world.regrow(true);
        }
        assert_eq!(
            world.standing_in(home, Cargo::Wood),
            summed(&world, Cargo::Wood),
            "and so does growing back"
        );
    }

    #[test]
    fn a_scouts_step_is_mostly_short_and_sometimes_very_long() {
        let draws: Vec<i32> = (0..20_000).map(|i| scout_step(WORLD_SEED, i)).collect();
        let over =
            |x: i32| draws.iter().filter(|step| **step > x).count() as f32 / draws.len() as f32;
        // For a tail going as one over the square, the share past x is the
        // shortest step over x. Two steps out is a half, ten is a tenth.
        assert!(
            (over(SCOUT_STEP_MIN * 2) - 0.5).abs() < 0.03,
            "half the steps should pass twice the shortest, got {}",
            over(SCOUT_STEP_MIN * 2)
        );
        assert!(
            (over(SCOUT_STEP_MIN * 10) - 0.1).abs() < 0.02,
            "a tenth should pass ten times it, got {}",
            over(SCOUT_STEP_MIN * 10)
        );
        assert!(
            draws
                .iter()
                .all(|step| *step >= SCOUT_STEP_MIN && *step <= SCOUT_STEP_MAX),
            "every step has to be inside its own bounds"
        );
        assert!(
            draws.iter().any(|step| *step > SCOUT_STEP_MIN * 20),
            "and the tail has to be heavy enough to actually reach"
        );
    }

    #[test]
    fn a_scout_walks_the_same_way_in_every_run() {
        let from = CENTER;
        for salt in 0..50 {
            assert_eq!(
                scout_target(from, WORLD_SEED, salt),
                scout_target(from, WORLD_SEED, salt),
                "a walk drawn from the seed has to repeat"
            );
        }
        let spread: Vec<IVec2> = (0..200)
            .map(|salt| scout_target(from, WORLD_SEED, salt))
            .collect();
        assert!(
            spread.iter().any(|to| to.x > from.x) && spread.iter().any(|to| to.x < from.x),
            "and it has to go both ways"
        );
        assert!(
            spread.iter().any(|to| to.y > from.y) && spread.iter().any(|to| to.y < from.y),
            "on both axes"
        );
    }

    #[test]
    fn a_founding_colony_knows_the_ground_it_lives_off_and_no_more() {
        let world = Patches::new(WORLD_SEED);
        for x in [-NEAR_GROUND, 0, NEAR_GROUND] {
            for y in [-NEAR_GROUND, 0, NEAR_GROUND] {
                let cell = CENTER + IVec2::new(x, y);
                assert!(
                    world.has_been_to(chunk_of(cell)),
                    "the colony must know the ground it works at {cell:?}"
                );
            }
        }
        assert!(
            !world.has_been_to(chunk_of(CENTER) + IVec2::new(2, 0)),
            "and must not know ground nobody has walked to"
        );
    }

    #[test]
    fn the_colony_only_works_ground_it_has_been_to() {
        let mut world = Patches::new(WORLD_SEED);
        let far = chunk_of(CENTER) + IVec2::new(2, 0);
        let inside = far * CHUNK + IVec2::splat(CHUNK / 2);
        assert!(!world.has_been_to(far));
        assert_eq!(
            world.nearest(Cargo::Wood, inside, CHUNK),
            None,
            "unknown ground holds nothing the colony can work"
        );
        world.discover(inside);
        assert!(world.has_been_to(far));
        assert!(
            world.nearest(Cargo::Wood, inside, CHUNK).is_some(),
            "and once somebody has stood there, it does"
        );
    }

    #[test]
    fn the_colony_never_forgets_where_it_has_been() {
        let mut world = Patches::new(WORLD_SEED);
        let far = chunk_of(CENTER) + IVec2::new(3, 1);
        let inside = far * CHUNK + IVec2::splat(CHUNK / 2);
        world.discover(inside);
        world.forget_beyond(&[], 0);
        assert!(
            world.has_been_to(far),
            "dropping a chunk from memory must not unlearn it"
        );
    }

    #[test]
    fn a_colony_spares_hands_for_looking_only_once_it_has_them() {
        assert_eq!(scouts_wanted(0), 0);
        assert_eq!(scouts_wanted(CITIZENS), 1, "a founding party spares one");
        assert!(
            scouts_wanted(CITIZENS * 4) > scouts_wanted(CITIZENS),
            "and a bigger colony spares more"
        );
        assert!(
            scouts_wanted(1000) <= 1000 / 10,
            "but never enough to stop being a colony that works"
        );
    }

    /// A second fire's output, for the tests that need one before anything in
    /// the colony can build one.
    const OUTPOST_HEAT: f32 = 40.0;

    fn one_fire(output: f32, ambient: f32) -> Air {
        Air {
            fires: vec![Fire { at: CENTER, output }],
            ambient,
        }
    }

    #[test]
    fn a_second_fire_warms_its_own_square_and_nothing_takes_that_away() {
        let ambient = AMBIENT_MEAN;
        let outpost = CENTER + IVec2::new(90, 0);
        let alone = one_fire(GENERATOR_HEAT, ambient);
        assert!(alone.heat_at(outpost) < 0.0, "nobody is warm out there yet");

        let mut lit = alone.clone();
        lit.fires.push(Fire {
            at: outpost,
            output: OUTPOST_HEAT,
        });
        assert!(lit.heat_at(outpost) > 0.0, "and now somebody is");
        assert_eq!(
            lit.heat_at(CENTER),
            alone.heat_at(CENTER),
            "a fire out there must not change the hearth"
        );
        for x in -30..=30 {
            let cell = CENTER + IVec2::new(x, 0);
            assert!(
                lit.heat_at(cell) >= alone.heat_at(cell),
                "a second fire can only ever add warmth"
            );
        }
    }

    #[test]
    fn air_with_nothing_lit_is_just_the_weather() {
        let air = Air {
            fires: Vec::new(),
            ambient: AMBIENT_MEAN,
        };
        assert_eq!(air.heat_at(CENTER), AMBIENT_MEAN);
        assert_eq!(air.heat_at(CENTER + IVec2::new(200, 200)), AMBIENT_MEAN);
    }

    #[test]
    fn the_walk_home_is_to_the_nearest_warmth_and_not_to_the_hearth() {
        let ambient = AMBIENT_MEAN;
        let outpost = CENTER + IVec2::new(90, 0);
        let alone = one_fire(GENERATOR_HEAT, ambient);
        let far = cost_of_getting_home(outpost, &alone);
        assert!(
            far > 70.0,
            "without a fire out there the walk is the whole way"
        );

        let mut lit = alone.clone();
        lit.fires.push(Fire {
            at: outpost,
            output: OUTPOST_HEAT,
        });
        assert_eq!(
            cost_of_getting_home(outpost, &lit),
            0.0,
            "standing at a lit waystation costs nothing to get warm at"
        );
        assert_eq!(
            cost_of_getting_home(CENTER, &lit),
            cost_of_getting_home(CENTER, &alone),
            "and the hearth is unaffected by what stands ninety cells away"
        );
    }

    #[test]
    fn a_citizen_walks_to_whichever_fire_is_nearer() {
        let ambient = AMBIENT_MEAN;
        let outpost = CENTER + IVec2::new(90, 0);
        let mut lit = one_fire(GENERATOR_HEAT, ambient);
        lit.fires.push(Fire {
            at: outpost,
            output: OUTPOST_HEAT,
        });
        let home = CENTER + IVec2::new(3, 3);
        assert_eq!(lit.warmth_target(home, outpost + IVec2::new(4, 0)), outpost);
        assert_eq!(lit.warmth_target(home, CENTER + IVec2::new(4, 0)), CENTER);
    }

    #[test]
    fn a_dead_fire_is_not_walked_to() {
        let ambient = AMBIENT_MEAN;
        let outpost = CENTER + IVec2::new(90, 0);
        let mut cold = one_fire(GENERATOR_HEAT, ambient);
        cold.fires.push(Fire {
            at: outpost,
            output: 0.0,
        });
        let home = outpost + IVec2::new(2, 2);
        assert_eq!(
            cold.warmth_target(home, outpost),
            CENTER,
            "an unlit waystation is a place to stand, not a place to warm up, so the walk is the long one"
        );
        cold.fires[0].output = 0.0;
        assert_eq!(
            cold.warmth_target(home, outpost),
            home,
            "and with nothing lit anywhere there is only a roof to go to"
        );
    }

    /// A citizen who has walked back to the fire from each of these places.
    fn detoured_from(walks: &[IVec2]) -> Needs {
        let mut needs = Needs::newcomer();
        for at in walks {
            needs.spend(true, *at);
        }
        needs
    }

    #[test]
    fn a_citizen_who_has_not_walked_back_has_nowhere_to_put_a_fire() {
        assert_eq!(Needs::newcomer().detour_middle(), None);
        assert_eq!(vote_of(&Needs::newcomer()), None);
    }

    #[test]
    fn walks_that_all_start_in_one_place_ask_for_a_fire_there() {
        let out = CENTER + IVec2::new(BOILER_REACHES + 12, 0);
        let needs = detoured_from(&[out, out + IVec2::new(1, 1), out - IVec2::new(1, 1), out]);
        assert_eq!(
            needs.detour_middle(),
            Some(out),
            "the middle of those walks is where they were walked"
        );
        assert_eq!(vote_of(&needs), Some(Building::Waystation));
    }

    #[test]
    fn walks_that_start_all_around_the_hearth_ask_for_a_bigger_hearth() {
        let far = BOILER_REACHES + 12;
        let needs = detoured_from(&[
            CENTER + IVec2::new(far, 0),
            CENTER + IVec2::new(-far, 0),
            CENTER + IVec2::new(0, far),
            CENTER + IVec2::new(0, -far),
        ]);
        assert_eq!(
            vote_of(&needs),
            Some(Building::GeneratorUpgrade),
            "walks scattered around the fire are not answered by moving it"
        );
    }

    #[test]
    fn walks_from_inside_what_a_boiler_reaches_ask_for_the_boiler() {
        let near = CENTER + IVec2::new(BOILER_REACHES - 1, 0);
        let needs = detoured_from(&[near, near, near]);
        assert_eq!(
            vote_of(&needs),
            Some(Building::GeneratorUpgrade),
            "a bigger fire still covers this walk, so it is the cheaper answer"
        );
    }

    #[test]
    fn a_boiler_reaches_as_far_as_a_boiler_reaches() {
        let ambient = AMBIENT_MEAN;
        let lit = one_fire(generator_output(FULL_BURN_FUEL, 1), ambient);
        assert!(
            lit.heat_at(CENTER + IVec2::new(BOILER_REACHES, 0)) >= 0.0,
            "the mark has to be inside what an upgraded fire actually holds"
        );
        assert!(
            lit.heat_at(CENTER + IVec2::new(BOILER_REACHES + 1, 0)) < 0.0,
            "and one cell past it has to be outside"
        );
    }

    #[test]
    fn a_site_is_where_the_walks_that_asked_for_it_concentrate() {
        let east = CENTER + IVec2::new(40, 0);
        let west = CENTER + IVec2::new(-40, 0);
        let asking = vec![detoured_from(&[east, east, east]), detoured_from(&[west])];
        let site = waystation_site(&asking, &mean_day()).expect("somebody asked");
        assert!(
            (site - CENTER).x > 0,
            "three walks east against one west put the fire east, at {site:?}"
        );
        assert_eq!(
            waystation_site(&[Needs::newcomer()], &mean_day()),
            None,
            "nobody asking means nowhere to put one"
        );
    }

    #[test]
    fn dying_where_the_colony_can_see_is_not_the_same_as_not_coming_back() {
        assert!(!goes_missing(CENTER), "nobody vanishes at the hearth");
        assert!(!goes_missing(CENTER + IVec2::splat(NEAR_GROUND)));
        assert!(
            goes_missing(CENTER + IVec2::new(NEAR_GROUND + 1, 0)),
            "past the ground the colony works, it only knows they did not come back"
        );
    }

    #[test]
    fn a_body_found_closes_the_event_and_nothing_else_does() {
        let out = CENTER + IVec2::new(NEAR_GROUND + 30, 0);
        let mut missing = Missing::default();
        missing.lost(out, 100);
        assert_eq!(missing.count(), 1);
        missing.recover(&[CENTER]);
        assert_eq!(missing.count(), 1, "nobody has been out there");
        missing.recover(&[out + IVec2::new(1, 0)]);
        assert_eq!(missing.count(), 0, "somebody walked to the spot");
    }

    #[test]
    fn a_slab_closes_what_no_walk_did_and_costs_the_colony_to_raise() {
        let out = CENTER + IVec2::new(NEAR_GROUND + 30, 0);
        let mut missing = Missing::default();
        missing.lost(out, 100);
        let mut wood = MEMORIAL_WOOD_COST * 2;
        missing.raise_memorials(100 + MEMORIAL_AFTER - 1, &mut wood);
        assert_eq!(missing.count(), 1, "the colony waits before it gives up");
        assert_eq!(
            wood,
            MEMORIAL_WOOD_COST * 2,
            "and spends nothing while it waits"
        );

        missing.raise_memorials(100 + MEMORIAL_AFTER, &mut wood);
        assert_eq!(missing.count(), 0, "then it raises a slab");
        assert_eq!(wood, MEMORIAL_WOOD_COST, "which costs what a slab costs");
    }

    #[test]
    fn a_colony_that_cannot_afford_a_slab_keeps_waiting() {
        let out = CENTER + IVec2::new(NEAR_GROUND + 30, 0);
        let mut missing = Missing::default();
        missing.lost(out, 0);
        let mut wood = MEMORIAL_WOOD_COST - 1;
        missing.raise_memorials(MEMORIAL_AFTER * 4, &mut wood);
        assert_eq!(missing.count(), 1, "a slab it cannot pay for is not raised");
        assert_eq!(wood, MEMORIAL_WOOD_COST - 1);
    }

    #[test]
    fn somebody_looking_goes_to_the_last_place_anybody_was_seen() {
        let out = CENTER + IVec2::new(80, 0);
        let mut missing = Missing::default();
        missing.lost(out, 0);
        assert_eq!(
            missing.nearest_to(CENTER),
            Some(out),
            "a walk with a body at the end of it beats a walk drawn from a hat"
        );
        assert_eq!(Missing::default().nearest_to(CENTER), None);
    }

    #[test]
    fn a_claimed_hauler_takes_their_load_to_the_shed_and_not_the_hearth() {
        let post = CENTER + IVec2::new(40, 0);
        assert_eq!(
            delivery_target(Cargo::Wood, true, None, Some(post)),
            post,
            "a shed that claimed somebody gets what they were carrying"
        );
        assert_eq!(
            delivery_target(Cargo::Wood, false, None, Some(post)),
            CENTER,
            "a colony with nothing to spare keeps its wood for the fire"
        );
        assert_eq!(
            delivery_target(Cargo::Food, true, None, Some(post)),
            CENTER,
            "a shed burns wood and not game"
        );
        let site = CENTER + IVec2::new(2, 0);
        assert_eq!(
            delivery_target(Cargo::Wood, true, Some(site), Some(post)),
            site,
            "and a project under way comes before a shed"
        );
    }

    /// A citizen with every need at the mark it counts as met at.
    fn contented() -> Needs {
        let mut needs = Needs::newcomer();
        for kind in NEEDS {
            set(&mut needs, kind, kind.rules().comfort, false);
        }
        needs
    }

    fn all_at(level: f32) -> Needs {
        let mut needs = Needs::newcomer();
        for kind in NEEDS {
            set(&mut needs, kind, level, level <= kind.rules().low);
        }
        needs
    }

    #[test]
    fn a_bored_citizen_can_still_be_spared_to_go_looking() {
        let mut needs = Needs::newcomer();
        needs.needs[NeedKind::Recreation as usize].burden = 100.0;
        assert!(
            needs.nothing_failed(),
            "boredom is not something the colony failed to answer"
        );
        for kind in NEEDS.into_iter().filter(|kind| kind.is_survival()) {
            let mut needs = Needs::newcomer();
            needs.needs[kind as usize].burden = 0.1;
            assert!(
                !needs.nothing_failed(),
                "{kind:?} going unanswered still keeps somebody home"
            );
        }
    }

    #[test]
    fn a_colony_with_nothing_to_do_of_an_evening_still_has_children() {
        let mut shares = [1.0; NEED_COUNT];
        shares[NeedKind::Recreation as usize] = 0.0;
        assert!(
            colony_thrives(shares, u32::MAX, u32::MAX, 30),
            "a comfort need going unmet is not a reason for a colony to stop"
        );
        for kind in NEEDS.into_iter().filter(|kind| kind.is_survival()) {
            let mut shares = [1.0; NEED_COUNT];
            shares[kind as usize] = 0.0;
            assert!(
                !colony_thrives(shares, u32::MAX, u32::MAX, 30),
                "{kind:?} going unmet across the colony still stops it"
            );
        }
    }

    #[test]
    fn being_bored_costs_a_citizen_nothing_at_the_work() {
        let mut bored = contented();
        set(&mut bored, NeedKind::Recreation, 0.0, true);
        assert_eq!(
            focus_of(&bored),
            focus_of(&contented()),
            "a comfort need is not what stops somebody working"
        );
        let mut cold = contented();
        set(&mut cold, NeedKind::Warmth, 0.0, true);
        let mut cold_and_bored = cold;
        set(&mut cold_and_bored, NeedKind::Recreation, 0.0, true);
        assert_eq!(
            focus_of(&cold_and_bored),
            focus_of(&cold),
            "and it does not dilute what does, either"
        );
        assert!(focus_of(&cold) < focus_of(&contented()));
    }

    #[test]
    fn a_citizen_with_nothing_the_matter_is_at_full_focus() {
        assert_eq!(focus_of(&contented()), 1.0, "nothing unmet, nothing lost");
        assert_eq!(
            focus_of(&all_at(NEED_MAX)),
            1.0,
            "and being better than met buys nothing -- focus is a ceiling"
        );
    }

    #[test]
    fn focus_never_leaves_its_own_range() {
        for level in 0..=100 {
            let focus = focus_of(&all_at(level as f32));
            assert!(
                (FOCUS_FLOOR..=1.0).contains(&focus),
                "focus {focus} at level {level} is outside its range"
            );
        }
        assert_eq!(
            focus_of(&all_at(0.0)),
            FOCUS_FLOOR,
            "the worst is the floor"
        );
    }

    #[test]
    fn focus_falls_without_a_step_in_it() {
        let mut previous = focus_of(&all_at(NEED_MAX));
        let mut biggest = 0.0f32;
        for level in (0..=100).rev() {
            let focus = focus_of(&all_at(level as f32));
            assert!(
                focus <= previous + f32::EPSILON,
                "focus rose as a need fell"
            );
            biggest = biggest.max(previous - focus);
            previous = focus;
        }
        assert!(
            biggest < (1.0 - FOCUS_FLOOR) / 4.0,
            "one point of a need took {biggest} of focus, which is a step and not a slope"
        );
    }

    #[test]
    fn the_last_of_a_need_costs_more_than_the_first_of_it() {
        let comfort = NeedKind::Warmth.rules().comfort;
        let near_the_top = focus_of(&all_at(comfort)) - focus_of(&all_at(comfort - 10.0));
        let near_the_bottom = focus_of(&all_at(10.0)) - focus_of(&all_at(0.0));
        assert!(
            near_the_bottom > near_the_top * 1.5,
            "the knee has to bite: {near_the_bottom} against {near_the_top}"
        );
    }

    #[test]
    fn the_needs_that_kill_weigh_the_same_and_outweigh_the_one_that_does_not() {
        for kind in NEEDS {
            let weight = kind.rules().weight;
            if kind.rules().fatal || kind == NeedKind::Rest {
                assert_eq!(
                    weight, NEED_WEIGHT_SURVIVAL,
                    "{kind:?} belongs to the tier a colony survives on"
                );
            } else {
                assert!(
                    weight < NEED_WEIGHT_SURVIVAL,
                    "{kind:?} must not weigh what survival weighs"
                );
            }
        }
    }

    #[test]
    fn the_band_is_a_word_for_the_number_and_nothing_more() {
        assert_eq!(focus_band(1.0), Focus::Focused);
        assert_eq!(focus_band(FOCUS_FLOOR), Focus::BadlyDistracted);
        let mut seen = Vec::new();
        for step in 0..=100 {
            let focus = FOCUS_FLOOR + (1.0 - FOCUS_FLOOR) * step as f32 / 100.0;
            let band = focus_band(focus);
            if seen.last() != Some(&band) {
                seen.push(band);
            }
        }
        assert_eq!(seen.len(), 4, "four bands, walked bottom to top, each once");
    }

    fn nothing_the_matter() -> [bool; THOUGHT_COUNT] {
        thoughts_of(&contented(), &at_the_fire(), false, false, true, false)
    }

    #[test]
    fn a_citizen_with_nothing_the_matter_still_has_something_to_feel() {
        let held = nothing_the_matter();
        assert!(
            held[Thought::Plenty as usize],
            "a colony with something put by is a thing to be glad of"
        );
        assert!(!held[Thought::Cold as usize]);
        assert!(!held[Thought::Hungry as usize]);
        assert!(!held[Thought::Worn as usize]);
        assert!(!held[Thought::Missing as usize]);
        assert_eq!(
            mood_target(&held, false),
            MOOD_BASE + Thought::Plenty.weight(),
            "the target is the base plus what is actually being felt"
        );
    }

    #[test]
    fn every_thought_has_a_name_and_a_weight_of_its_own() {
        let mut names = Vec::new();
        for thought in THOUGHTS {
            assert!(!thought.name().is_empty(), "a thought nobody can print");
            assert!(!names.contains(&thought.name()), "two thoughts, one name");
            names.push(thought.name());
            assert_ne!(thought.weight(), 0.0, "a thought that changes nothing");
        }
        assert_eq!(names.len(), THOUGHT_COUNT);
    }

    #[test]
    fn what_is_wrong_shows_up_by_name_and_not_only_in_the_total() {
        let cold = thoughts_of(&all_at(1.0), &at_the_fire(), true, true, false, false);
        assert!(cold[Thought::Cold as usize]);
        assert!(cold[Thought::Hungry as usize]);
        assert!(cold[Thought::Worn as usize]);
        assert!(cold[Thought::WrongWork as usize]);
        assert!(cold[Thought::Missing as usize]);
        assert!(!cold[Thought::Plenty as usize]);
        assert!(
            mood_target(&cold, false) < mood_target(&nothing_the_matter(), false),
            "and the total follows the names rather than replacing them"
        );
    }

    #[test]
    fn a_mood_rises_faster_than_it_falls() {
        let rise = mood_step(MOOD_BASE, MOOD_BASE + 40.0, false);
        let fall = mood_step(MOOD_BASE, MOOD_BASE - 40.0, false);
        assert!(rise > MOOD_BASE && fall < MOOD_BASE);
        assert!(
            rise - MOOD_BASE > MOOD_BASE - fall,
            "cheering up is quicker than sinking, which is the shape borrowed"
        );
    }

    #[test]
    fn a_mood_does_not_move_while_somebody_is_asleep() {
        assert_eq!(mood_step(40.0, 90.0, true), 40.0);
        assert_eq!(mood_step(40.0, 0.0, true), 40.0);
    }

    #[test]
    fn a_mood_settles_on_its_target_and_stops_there() {
        let target = MOOD_BASE - 30.0;
        let mut mood = MOOD_BASE;
        for _ in 0..ticks_per_day() * 30 {
            mood = mood_step(mood, target, false);
        }
        assert!(
            (mood - target).abs() < 0.5,
            "a month of chasing should arrive: {mood} against {target}"
        );
        assert_eq!(mood_step(target, target, false), target, "and then stay");
    }

    #[test]
    fn a_mood_never_leaves_the_range_it_is_printed_in() {
        for target in [-500.0, 0.0, MOOD_MAX, 500.0] {
            let mut mood = MOOD_BASE;
            for _ in 0..ticks_per_year() {
                mood = mood_step(mood, target, false);
                assert!(
                    (0.0..=MOOD_MAX).contains(&mood),
                    "mood {mood} is off the scale"
                );
            }
        }
    }

    #[test]
    fn a_bad_year_leaves_a_mark_and_a_good_one_does_not() {
        assert!(
            hardship_step(0.0, HARDSHIP_MARK - 1.0) > 0.0,
            "misery accumulates"
        );
        assert_eq!(
            hardship_step(0.0, HARDSHIP_EASE + 1.0),
            0.0,
            "and contentment has nothing to take away yet"
        );
        assert!(
            hardship_step(50.0, HARDSHIP_EASE + 1.0) < 50.0,
            "but it does take away what is there"
        );
    }

    #[test]
    fn between_the_marks_a_mark_neither_deepens_nor_fades() {
        let middle = (HARDSHIP_MARK + HARDSHIP_EASE) / 2.0;
        assert_eq!(hardship_step(50.0, middle), 50.0);
        // The gap between the two marks is what stops it flickering, per ADR 0003.
        const _: () = assert!(HARDSHIP_EASE > HARDSHIP_MARK);
    }

    #[test]
    fn what_a_bad_decade_does_takes_a_worse_one_to_undo() {
        let gained = hardship_step(0.0, 0.0);
        let faded = 50.0 - hardship_step(50.0, MOOD_MAX);
        assert!(
            gained > faded * 3.0,
            "a mark has to be made faster than it is unmade: {gained} against {faded}"
        );
    }

    #[test]
    fn a_mark_takes_years_to_make_and_longer_to_lose() {
        let mut hardship = 0.0;
        let mut years = 0;
        while hardship < HARDSHIP_MAX && years < 100 {
            for _ in 0..ticks_per_year() {
                hardship = hardship_step(hardship, 0.0);
            }
            years += 1;
        }
        assert!(
            (2..=6).contains(&years),
            "a life of misery should mark somebody in a few years, took {years}"
        );
        let mut clearing = 0;
        while hardship > 0.0 && clearing < 100 {
            for _ in 0..ticks_per_year() {
                hardship = hardship_step(hardship, MOOD_MAX);
            }
            clearing += 1;
        }
        assert!(
            clearing > years * 2,
            "and losing it should take far longer: {clearing} against {years}"
        );
    }

    #[test]
    fn a_mark_never_leaves_its_own_range() {
        let mut hardship = 0.0;
        for _ in 0..ticks_per_year() * 40 {
            hardship = hardship_step(hardship, 0.0);
            assert!((0.0..=HARDSHIP_MAX).contains(&hardship));
        }
        for _ in 0..ticks_per_year() * 80 {
            hardship = hardship_step(hardship, MOOD_MAX);
            assert!((0.0..=HARDSHIP_MAX).contains(&hardship));
        }
    }

    #[test]
    fn every_depth_of_a_mark_has_a_word_for_it() {
        assert_eq!(hardship_status(0.0), Hardship::Untouched);
        assert_eq!(hardship_status(HARDSHIP_MAX), Hardship::Broken);
        let mut seen = Vec::new();
        for step in 0..=100 {
            let status = hardship_status(HARDSHIP_MAX * step as f32 / 100.0);
            if seen.last() != Some(&status) {
                seen.push(status);
            }
            assert!(!status.name().is_empty());
        }
        assert_eq!(seen.len(), 4, "four words, walked bottom to top, each once");
    }

    /// The air on one particular hour of one particular day.
    fn air_on(day: u64, hour: u64) -> f32 {
        ambient_at(day * ticks_per_day() + hour * TICKS_PER_HOUR)
    }

    #[test]
    fn the_night_is_colder_than_the_day_all_year_round() {
        for day in 0..days_per_year() {
            let night = air_on(day, COLDEST_HOUR);
            let noon = air_on(day, (COLDEST_HOUR + HOURS_PER_DAY / 2) % HOURS_PER_DAY);
            assert!(
                noon > night,
                "day {day}: noon {noon} is not warmer than the small hours {night}"
            );
        }
    }

    #[test]
    fn the_coldest_and_warmest_hours_are_the_named_ones() {
        let day = days_per_year() / 3;
        let hours: Vec<f32> = (0..HOURS_PER_DAY).map(|hour| air_on(day, hour)).collect();
        let coldest = hours
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(hour, _)| hour as u64)
            .expect("a day has hours in it");
        assert_eq!(coldest, COLDEST_HOUR);
        let warmest = hours
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(hour, _)| hour as u64)
            .expect("a day has hours in it");
        assert_eq!(warmest, (COLDEST_HOUR + HOURS_PER_DAY / 2) % HOURS_PER_DAY);
    }

    #[test]
    fn a_day_swings_by_exactly_what_it_is_said_to() {
        let day = days_per_year() / 3;
        let hours: Vec<f32> = (0..HOURS_PER_DAY).map(|hour| air_on(day, hour)).collect();
        let low = hours.iter().copied().fold(f32::MAX, f32::min);
        let high = hours.iter().copied().fold(f32::MIN, f32::max);
        assert!(
            (high - low - DIURNAL_SWING).abs() < 0.01,
            "the day swung {} where it should swing {DIURNAL_SWING}",
            high - low
        );
    }

    #[test]
    fn a_whole_day_averages_out_to_the_climate_it_sits_on() {
        for day in [0, 17, days_per_year() / 2, days_per_year() - 1] {
            let mean: f32 = (0..HOURS_PER_DAY)
                .map(|hour| air_on(day, hour))
                .sum::<f32>()
                / HOURS_PER_DAY as f32;
            assert!(
                (mean - climate_at(day * ticks_per_day())).abs() < 0.01,
                "day {day} averaged {mean}, which is not the climate under it"
            );
        }
    }

    #[test]
    fn the_climate_underneath_is_the_curve_it_always_was() {
        // The year's shape is out of scope for the hour that rides on it: this
        // pins that nothing about the seasons moved when the days got a shape.
        let midwinter = day_of(
            SEVERITY_FULL_YEAR,
            DAYS_PER_SEASON * 3 + DAYS_PER_SEASON / 2,
        );
        let midsummer = day_of(SEVERITY_FULL_YEAR, DAYS_PER_SEASON + DAYS_PER_SEASON / 2);
        assert!(climate_at(midwinter) < AMBIENT_MEAN);
        assert!(climate_at(midsummer) > AMBIENT_MEAN);
    }

    /// The front on each of a run of days, from a still start.
    fn front_over(days: u64) -> Vec<f32> {
        let mut front = 0.0;
        (0..days)
            .map(|day| {
                front = front_step(front, WORLD_SEED, day);
                front
            })
            .collect()
    }

    #[test]
    fn a_front_fades_over_the_days_it_says_it_does() {
        // Measured off the weather the colony actually gets rather than off the
        // arithmetic that makes it: how much of a day's departure is still
        // there a correlation time later, averaged over twenty years.
        let days = front_over(days_per_year() * 20);
        let lag = FRONT_DAYS as usize;
        let power: f32 = days.iter().map(|f| f * f).sum::<f32>() / days.len() as f32;
        let carried: f32 = days
            .windows(lag + 1)
            .map(|window| window[0] * window[lag])
            .sum::<f32>()
            / (days.len() - lag) as f32;
        let left = carried / power;
        assert!(
            (left - 1.0 / std::f32::consts::E).abs() < 0.1,
            "a front should be down to about a third after {FRONT_DAYS} days, it is at {left}"
        );
    }

    #[test]
    fn a_front_never_takes_the_air_further_than_it_is_allowed() {
        for front in front_over(ticks_per_year() / ticks_per_day() * 20) {
            assert!(
                front.abs() <= FRONT_CAP + f32::EPSILON,
                "a front reached {front}, past a cap of {FRONT_CAP}"
            );
        }
    }

    #[test]
    fn a_front_wanders_both_ways_and_comes_back_to_the_season() {
        let days = front_over(days_per_year() * 20);
        assert!(
            days.iter().any(|f| *f > 2.0),
            "no warm spell in twenty years"
        );
        assert!(days.iter().any(|f| *f < -2.0), "no cold snap either");
        let mean = days.iter().sum::<f32>() / days.len() as f32;
        assert!(
            mean.abs() < 0.5,
            "over twenty years the weather should average to the season, not {mean}"
        );
    }

    #[test]
    fn today_looks_more_like_yesterday_than_like_last_week() {
        let days = front_over(days_per_year() * 10);
        let gap = |apart: usize| -> f32 {
            days.windows(apart + 1)
                .map(|w| (w[apart] - w[0]).abs())
                .sum::<f32>()
                / (days.len() - apart) as f32
        };
        assert!(
            gap(1) < gap(7),
            "a day apart should differ less than a week apart: {} against {}",
            gap(1),
            gap(7)
        );
    }

    #[test]
    fn the_same_world_gets_the_same_weather_every_run() {
        assert_eq!(front_over(500), front_over(500));
        let other: Vec<f32> = {
            let mut front = 0.0;
            (0..500)
                .map(|day| {
                    front = front_step(front, WORLD_SEED ^ 0xff, day);
                    front
                })
                .collect()
        };
        assert_ne!(
            front_over(500),
            other,
            "a different world gets its own weather"
        );
    }

    fn well_off() -> (usize, u32, u32) {
        (CITIZENS * 2, 4000, 2000)
    }

    #[test]
    fn a_colony_with_more_to_lose_can_be_hit_harder() {
        let (pop, fuel, food) = well_off();
        let fat = severity_budget(pop, fuel, food, SEVERITY_FULL_YEAR, 1.0);
        // Fewer mouths and, the part that matters, less per mouth: dividing the
        // stores by the same factor as the heads leaves a colony just as well
        // off and buys nothing.
        let lean = severity_budget(pop / 4, fuel / 200, food / 200, SEVERITY_FULL_YEAR, 1.0);
        assert!(
            fat > lean * 1.5,
            "a fat colony should be worth hitting harder: {fat} against {lean}"
        );
        assert!(lean >= 0.0, "and a poor one is never owed a bonus");
    }

    #[test]
    fn nothing_the_colony_does_gets_it_hit_past_the_ceiling() {
        for pop in [1, CITIZENS, 500, 5000] {
            for stores in [0, 1000, u32::MAX / 2] {
                let budget = severity_budget(pop, stores, stores, 900, ADAPT_MAX);
                assert!(
                    (0.0..=BUDGET_CEILING).contains(&budget),
                    "pop {pop} with {stores} put by bought a budget of {budget}"
                );
            }
        }
    }

    #[test]
    fn the_first_year_is_held_down_and_the_fortieth_is_not() {
        let (pop, fuel, food) = well_off();
        let first = severity_budget(pop, fuel, food, 1, 1.0);
        let later = severity_budget(pop, fuel, food, SEVERITY_FULL_YEAR, 1.0);
        assert!(
            first < later,
            "the ramp that holds the first winter down is the same ramp"
        );
    }

    #[test]
    fn grace_is_bought_by_time_and_spent_by_a_death() {
        let mut grace = ADAPT_MIN;
        for _ in 0..ticks_per_year() * 5 {
            grace = adaptation_step(grace, false);
        }
        assert!(grace > ADAPT_MIN, "five quiet years should buy something");
        let after = adaptation_step(grace, true);
        assert!(after < grace, "and a death should spend it");
        let mut floor = ADAPT_MAX;
        for _ in 0..200 {
            floor = adaptation_step(floor, true);
        }
        assert!(
            (ADAPT_MIN..=ADAPT_MAX).contains(&floor),
            "grace stayed inside its bounds: {floor}"
        );
    }

    #[test]
    fn a_spell_is_seen_coming_before_it_lands() {
        let spell = Weathering {
            kind: Spell::Blizzard,
            depth: 9.0,
            began: 100,
            days: 10,
        };
        assert_eq!(spell.air_on(99), 0.0, "nothing before it starts");
        assert_eq!(spell.air_on(100), 0.0, "nor on the day it turns");
        let onset = spell.air_on(100 + SPELL_ONSET_DAYS / 2);
        assert!(
            onset < 0.0 && onset > -9.0,
            "the approach has to be visible and shallower than the depth: {onset}"
        );
        assert_eq!(spell.air_on(100 + SPELL_ONSET_DAYS), -9.0, "then the depth");
        assert_eq!(spell.air_on(110), 0.0, "and nothing after it is over");
    }

    #[test]
    fn a_thaw_goes_the_other_way() {
        let thaw = Weathering {
            kind: Spell::Thaw,
            depth: 5.0,
            began: 0,
            days: 8,
        };
        assert!(
            thaw.air_on(SPELL_ONSET_DAYS) > 0.0,
            "a thaw is warmth, not cold"
        );
    }

    #[test]
    fn the_colony_has_a_word_for_what_the_sky_is_doing() {
        assert_eq!(weather_word(None, 0), "clear");
        let blizzard = Weathering {
            kind: Spell::Blizzard,
            depth: 9.0,
            began: 0,
            days: 10,
        };
        assert_eq!(weather_word(Some(&blizzard), SPELL_ONSET_DAYS), "blizzard");
        let snap = Weathering {
            kind: Spell::ColdSnap,
            depth: 4.0,
            began: 0,
            days: 10,
        };
        assert_eq!(weather_word(Some(&snap), SPELL_ONSET_DAYS), "snow");
        let thaw = Weathering {
            kind: Spell::Thaw,
            depth: 4.0,
            began: 0,
            days: 10,
        };
        assert_eq!(weather_word(Some(&thaw), SPELL_ONSET_DAYS), "thaw");
        assert_eq!(
            weather_word(Some(&blizzard), 99),
            "clear",
            "a spell that is over is not a word any more"
        );
    }

    #[test]
    fn a_bigger_budget_buys_deeper_and_more_frequent_weather() {
        let over_a_decade = |budget: f32| -> (usize, f32) {
            let spells: Vec<Weathering> = (0..days_per_year() * 10)
                .filter_map(|day| spell_due(budget, WORLD_SEED, day))
                .collect();
            let deepest = spells.iter().map(|s| s.depth).fold(0.0f32, f32::max);
            (spells.len(), deepest)
        };
        let (rare, shallow) = over_a_decade(BUDGET_CEILING / 4.0);
        let (often, deep) = over_a_decade(BUDGET_CEILING);
        assert!(
            often > rare,
            "a richer colony sees more of them: {often} against {rare}"
        );
        assert!(deep > shallow, "and deeper ones: {deep} against {shallow}");
        assert!(deep <= BUDGET_CEILING, "but never past the ceiling");
    }

    #[test]
    fn a_survivor_feels_the_same_cold_less() {
        let bad = thoughts_of(&all_at(1.0), &at_the_fire(), true, true, false, false);
        let ordinary = mood_target(&bad, false);
        let spared = mood_target(&bad, true);
        assert!(
            spared > ordinary,
            "the same winter should weigh less on somebody who has just buried a quarter of the colony: {spared} against {ordinary}"
        );
        assert!(spared < MOOD_BASE, "it weighs less, not nothing");
    }

    #[test]
    fn what_is_going_right_is_worth_the_same_to_everybody() {
        let good = thoughts_of(&contented(), &at_the_fire(), false, false, true, false);
        assert_eq!(
            mood_target(&good, true),
            mood_target(&good, false),
            "expectations falling is about what hurts, never about what helps"
        );
    }

    #[test]
    fn a_season_has_to_take_a_share_before_it_changes_anybody() {
        let began = 40;
        assert!(!season_broke_them(
            began,
            (began as f32 * LOSS_SHARE) as u64 - 1
        ));
        assert!(season_broke_them(
            began,
            (began as f32 * LOSS_SHARE).ceil() as u64
        ));
        assert!(
            !season_broke_them(0, 0),
            "a colony nobody was left in did not have a bad winter, it ended"
        );
    }

    #[test]
    fn expectations_come_back_up_after_the_seasons_they_are_given() {
        let began = 1_000;
        let until = spared_until(began);
        assert_eq!(
            until,
            began + EXPECTATIONS_SEASONS * ticks_per_season(),
            "it lasts the seasons it says it lasts"
        );
        assert!(until > began);
    }

    #[test]
    fn wanting_to_be_amused_never_outranks_wanting_to_live() {
        assert!(
            NeedKind::Recreation.rules().weight < NEED_WEIGHT_SURVIVAL,
            "recreation is not in the tier that kills"
        );
        assert!(
            !NeedKind::Recreation.rules().fatal,
            "and nobody dies of never being amused"
        );
        let mut needs = Needs::newcomer();
        // Bored past bearing and hungry only just: hunger still wins.
        set(&mut needs, NeedKind::Recreation, 0.0, true);
        set(
            &mut needs,
            NeedKind::Food,
            NeedKind::Food.rules().low - 0.5,
            true,
        );
        assert_eq!(
            needs.pressing_by_urgency(&at_the_fire())[0],
            NeedKind::Food,
            "a survival need outranks recreation whatever the shortfalls say"
        );
    }

    #[test]
    fn three_needs_have_a_building_for_a_remedy_and_the_fourth_has_a_person() {
        let mut answered = Vec::new();
        for kind in NEEDS {
            match building_for(kind) {
                Some(building) => {
                    assert!(!answered.contains(&building), "{kind:?} shares a remedy");
                    answered.push(building);
                }
                None => assert_eq!(
                    kind,
                    NeedKind::Recreation,
                    "{kind:?} has nothing that answers it"
                ),
            }
        }
        assert_eq!(answered.len(), NEED_COUNT - 1);
        // What is left over is elected by the other tier of the ballot: nothing
        // going wrong asks for a waystation, only hours going to waste do.
        let spare: Vec<Building> = BUILDINGS
            .into_iter()
            .filter(|building| !answered.contains(building))
            .collect();
        assert_eq!(spare, vec![Building::Waystation]);
    }

    #[test]
    fn nothing_the_colony_can_build_answers_being_bored() {
        let mut needs = Needs::newcomer();
        set(&mut needs, NeedKind::Recreation, 0.0, true);
        needs.needs[NeedKind::Recreation as usize].burden = 40.0;
        assert_eq!(
            vote_of(&needs),
            None,
            "a citizen with nothing wrong but boredom asks the ballot for nothing"
        );
    }

    #[test]
    fn a_performance_is_worth_what_it_is_made_of() {
        let bare = performance_quality(0, -10.0, 0.2, 0.0);
        let full = performance_quality(12, 5.0, 0.9, 1.0);
        assert!(full > bare, "every term has to be able to move it");
        assert!((0.0..=1.0).contains(&bare) && (0.0..=1.0).contains(&full));
        assert!(
            performance_quality(6, 5.0, 0.9, 1.0) > performance_quality(1, 5.0, 0.9, 1.0),
            "an audience is one of the terms"
        );
        assert!(
            performance_quality(6, 5.0, 0.9, 1.0) > performance_quality(6, -20.0, 0.9, 1.0),
            "so is the warmth where they are standing"
        );
        assert!(
            performance_quality(6, 5.0, 0.9, 1.0) > performance_quality(6, 5.0, 0.1, 1.0),
            "and so is the performer"
        );
    }

    #[test]
    fn a_better_performance_more_often_goes_well() {
        let over_many = |quality: f32| -> f32 {
            let good = (0..2000)
                .filter(|i| {
                    matches!(
                        performance_outcome(quality, WORLD_SEED, *i as u64),
                        Outcome::Fun | Outcome::Unforgettable
                    )
                })
                .count();
            good as f32 / 2000.0
        };
        let poor = over_many(0.1);
        let fine = over_many(0.9);
        assert!(
            fine > poor * 1.5,
            "quality has to tell: {fine} against {poor}"
        );
        assert!(poor > 0.0, "and a poor night can still go well");
        assert!(fine < 1.0, "and a good one can still fall flat");
    }

    #[test]
    fn only_a_night_that_went_well_is_worth_anything() {
        assert!(Outcome::Unforgettable.mood_days() > Outcome::Fun.mood_days());
        assert_eq!(Outcome::Boring.mood_days(), 0);
        assert_eq!(Outcome::Terrible.mood_days(), 0);
        for outcome in [
            Outcome::Terrible,
            Outcome::Boring,
            Outcome::Fun,
            Outcome::Unforgettable,
        ] {
            assert!(!outcome.name().is_empty());
        }
    }
}
