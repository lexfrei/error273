use crate::sim::{
    ADULT_AGE, BUILDING_COUNT, BUILDINGS, Building, Calendar, FRAILTY_ONSET, couples, is_adult,
};

pub const STATUS_LINES: usize = 4;

/// Everything the status lines report. Both faces of the game fill one of these
/// before formatting, so a reading that exists in one and not the other is a
/// compile error rather than a drift nobody notices.
pub struct Status {
    pub tick: u64,
    pub calendar: Calendar,
    pub ambient: f32,
    pub alive: usize,
    pub fuel: u32,
    pub food: u32,
    pub wood: u32,
    pub game: u32,
    pub buildings: [usize; BUILDING_COUNT],
    /// The project in hand and how much timber it has swallowed, if any.
    pub project: Option<(Building, u32)>,
    pub tally: [f32; BUILDING_COUNT],
    pub ages: Vec<f32>,
}

/// The four lines under the map, and the whole of what the headless build
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
            "pop {:3}  fuel {:4}  food {:4}  wood {:4}  game {:4}",
            status.alive, status.fuel, status.food, status.wood, status.game
        ),
        format!(
            "{}  project {}  vote {}",
            counts.join("  "),
            project,
            votes.join("/")
        ),
        format!(
            "under {:<2} {:3}  grown {:3}  over {:<2} {:3}  couples {:3}",
            ADULT_AGE as u32,
            children,
            status.alive.saturating_sub(children + frail),
            FRAILTY_ONSET as u32,
            frail,
            couples(&status.ages)
        ),
    ]
}
