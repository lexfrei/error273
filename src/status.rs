use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::sim::{
    ADULT_AGE, BUILDING_COUNT, BUILDINGS, Ballot, Building, CENTER, Calendar, Cargo, Construction,
    FRAILTY_ONSET, Missing, NEAR_GROUND, Outside, Patches, Regard, STAT_COUNT, STATS, Stores,
    couples, is_adult,
};

pub const STATUS_LINES: usize = 5;

/// Everything a face reads to fill a `Status`, in one borrow. The two of them
/// ask the world the same questions, so they ask through the same param.
#[derive(SystemParam)]
pub struct Readings<'w> {
    pub outside: Outside<'w>,
    pub stores: Stores<'w>,
    pub patches: Res<'w, Patches>,
    pub construction: Res<'w, Construction>,
    pub ballot: Res<'w, Ballot>,
    pub missing: Res<'w, Missing>,
}

impl Readings<'_> {
    /// What is standing on the ground the colony lives off. The world has no
    /// edge, so a total over all of it would only report how far the colony has
    /// wandered; this is the near ground, which is where the work is.
    pub fn standing(&self, kind: Cargo) -> u32 {
        self.patches
            .seen(CENTER, NEAR_GROUND)
            .filter(|patch| patch.kind == kind)
            .map(|patch| patch.amount)
            .sum()
    }
}

/// Everything the status lines report. Both faces of the game fill one of these
/// before formatting, so a reading that exists in one and not the other is a
/// compile error rather than a drift nobody notices.
pub struct Status {
    pub tick: u64,
    pub calendar: Calendar,
    pub ambient: f32,
    pub alive: usize,
    /// Citizens working past the edge of the frame. The window cannot show them
    /// and the headless build has no frame at all, so both report the count.
    pub off_frame: usize,
    /// Citizens who went out and did not come back. Not a death toll: the
    /// colony does not know what happened, only that they are not here.
    pub missing: usize,
    pub fuel: u32,
    pub food: u32,
    pub wood: u32,
    pub game: u32,
    pub buildings: [usize; BUILDING_COUNT],
    /// The project in hand and how much timber it has swallowed, if any.
    pub project: Option<(Building, u32)>,
    pub tally: [f32; BUILDING_COUNT],
    pub ages: Vec<f32>,
    /// The middle of the colony for each stat. An aggregate, not a reveal: what
    /// any one citizen is stays hidden until the colony has watched them work.
    pub stats: [f32; STAT_COUNT],
    /// One citizen, described the way the colony would describe them.
    pub card: Option<CitizenCard>,
}

/// What the colony would say about one of its own. Words rather than numbers,
/// and the colony says plainly whether it is guessing.
pub struct CitizenCard {
    pub seed: u64,
    pub age: f32,
    pub words: [Regard; STAT_COUNT],
    pub watched: f32,
    pub known: bool,
}

/// The five lines under the map, and the whole of what the headless build
/// prints. Parsed by the balance log, so the shape of these is an interface.
pub fn status_lines(status: &Status) -> [String; STATUS_LINES] {
    let project = match status.project {
        Some((building, delivered)) => format!(
            "{} {}/{}",
            building.rules().name,
            delivered,
            building.rules().cost
        ),
        None => "none".to_string(),
    };
    let counts: Vec<String> = BUILDINGS
        .into_iter()
        .map(|building| {
            format!(
                "{} {:3}",
                building.rules().name,
                status.buildings[building as usize]
            )
        })
        .collect();
    let votes: Vec<String> = BUILDINGS
        .into_iter()
        .map(|building| format!("{:.0}", status.tally[building as usize]))
        .collect();
    let children = status.ages.iter().filter(|age| !is_adult(**age)).count();
    let frail = status
        .ages
        .iter()
        .filter(|age| **age > FRAILTY_ONSET)
        .count();
    [
        format!(
            "tick {:5}  year {}  {:<6}  day {:2}  hour {:02}  air {:+.0}",
            status.tick,
            status.calendar.year,
            status.calendar.season.name(),
            status.calendar.day,
            status.calendar.hour,
            status.ambient
        ),
        format!(
            "pop {:3}  out {:2}  missing {:2}  fuel {:4}  food {:4}  wood {:4}  game {:4}",
            status.alive,
            status.off_frame,
            status.missing,
            status.fuel,
            status.food,
            status.wood,
            status.game
        ),
        format!(
            "{}  project {}  vote {}",
            counts.join("  "),
            project,
            votes.join("/")
        ),
        format!(
            "under {:<2} {:3}  grown {:3}  over {:<2} {:3}  couples {:3}  raised {}",
            ADULT_AGE as u32,
            children,
            status.alive.saturating_sub(children + frail),
            FRAILTY_ONSET as u32,
            frail,
            couples(&status.ages),
            STATS
                .into_iter()
                .map(|stat| format!("{:.2}", status.stats[stat as usize]))
                .collect::<Vec<String>>()
                .join("/")
        ),
        match &status.card {
            Some(card) => format!(
                "eldest #{:<3} {:2.0}y  {}  watched {:3.0}d {}",
                card.seed,
                card.age,
                STATS
                    .into_iter()
                    .map(|stat| format!(
                        "{} {:<6}",
                        stat_word(stat),
                        card.words[stat as usize].word()
                    ))
                    .collect::<Vec<String>>()
                    .join(" "),
                card.watched,
                if card.known { "" } else { "(a guess)" }
            ),
            None => "eldest --".to_string(),
        },
    ]
}

fn stat_word(stat: crate::sim::Stat) -> &'static str {
    match stat {
        crate::sim::Stat::Strength => "str",
        crate::sim::Stat::Wits => "wit",
        crate::sim::Stat::Hardiness => "hard",
    }
}
