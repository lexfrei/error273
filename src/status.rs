use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::sim::{
    ADULT_AGE, BUILDING_COUNT, BUILDINGS, Ballot, Building, CENTER, Calendar, Cargo, Construction,
    FRAILTY_ONSET, Focus, Hardship, Missing, NEAR_GROUND, Outside, Patches, Regard, STAT_COUNT,
    STATS, Stores, THOUGHT_COUNT, THOUGHTS, Thought, Weather, couples, is_adult,
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
    pub weather: Res<'w, Weather>,
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
    /// What the sky is doing, in the colony's own word for it. Beside the air
    /// rather than instead of it: the number says how cold and the word says
    /// why.
    pub weather: &'static str,
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
    /// The middle of the colony, the same kind of aggregate as the stats beside
    /// it: what any one citizen feels stays on their own card.
    pub mood: f32,
    /// How the last performance went, in the colony's own word.
    pub revel: &'static str,
    /// What that night was, which is what a colony running out of things to put
    /// on looks like from the outside.
    pub revel_kind: &'static str,
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
    /// How much of themselves this one has to give the work today. Unlike the
    /// three stats it is not an estimate and not hidden: anybody watching a
    /// citizen can see they are distracted.
    pub focus: Focus,
    /// How they are bearing up, and what they are holding that makes them.
    /// Printed rather than acted on: nothing in the colony reads a mood yet.
    pub mood: f32,
    pub held: [bool; THOUGHT_COUNT],
    /// What the years have done to them, which the mood has already stopped
    /// explaining. Printed as a word, like the focus band.
    pub hardship: Hardship,
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
            "tick {:5}  year {}  {:<6}  day {:2}  hour {:02}  air {:+.0} {}",
            status.tick,
            status.calendar.year,
            status.calendar.season.name(),
            status.calendar.day,
            status.calendar.hour,
            status.ambient,
            status.weather
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
            "under {:<2} {:3}  grown {:3}  over {:<2} {:3}  couples {:3}  raised {}  mood {:3.0}  night {} {}",
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
                .join("/"),
            status.mood,
            status.revel_kind,
            status.revel
        ),
        match &status.card {
            Some(card) => format!(
                "eldest #{:<3} {:2.0}y  {}  {:<16} mood {:3.0} {:<9} {:<28} watched {:3.0}d {}",
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
                card.focus.name(),
                card.mood,
                card.hardship.name(),
                THOUGHTS
                    .into_iter()
                    .filter(|thought| card.held[*thought as usize])
                    .map(Thought::name)
                    .collect::<Vec<&str>>()
                    .join(", "),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{MOOD_BASE, MOOD_MAX, calendar_at};

    /// The page carries a sample of these lines, and a sample is a copy that
    /// nothing keeps in step. Three times now a reading has been added here and
    /// the copy has gone on printing the old shape, caught by somebody reading
    /// rather than by anything checking. This checks it: every word the lines
    /// print has to appear in the page's sample, so a new reading either gets
    /// into the sample or fails here.
    #[test]
    fn the_page_shows_every_reading_the_build_prints() {
        let page = include_str!("../README.md");
        // To the closing fence and no further. Splitting on the opening one and
        // taking what follows hands back the whole rest of the page, prose and
        // all, so a word can be found in an unrelated sentence and the check
        // passes on a sample that has not said it for two changes.
        let sample = page
            .split("```text")
            .nth(1)
            .and_then(|rest| rest.split("```").next())
            .expect("the page has a sample of a run in it");
        let shown = Status {
            tick: 0,
            calendar: calendar_at(0),
            ambient: 0.0,
            weather: "clear",
            alive: 1,
            off_frame: 0,
            missing: 0,
            fuel: 0,
            food: 0,
            wood: 0,
            game: 0,
            buildings: [0; BUILDING_COUNT],
            project: None,
            tally: [0.0; BUILDING_COUNT],
            ages: vec![30.0],
            stats: [0.5; STAT_COUNT],
            mood: MOOD_BASE,
            revel: "fun",
            revel_kind: "song",
            card: Some(CitizenCard {
                seed: 1,
                age: 30.0,
                words: [Regard::Middling; STAT_COUNT],
                watched: 0.0,
                known: true,
                focus: Focus::Focused,
                mood: MOOD_BASE,
                held: [true; THOUGHT_COUNT],
                hardship: Hardship::Untouched,
            }),
        };
        // Only the words the format itself puts there, never the ones that came
        // from the data: a sample catches whichever citizen it catches, and
        // requiring it to say `focused` would fail the day it catches somebody
        // distracted. What the two differ in is data; what they share is label.
        let words = |status: &Status| -> Vec<String> {
            status_lines(status)
                .into_iter()
                .flat_map(|line| {
                    line.split_whitespace()
                        .filter(|word| word.chars().all(|c| c.is_ascii_lowercase()))
                        .map(str::to_string)
                        .collect::<Vec<String>>()
                })
                .collect()
        };
        let mut other = Status {
            ages: shown.ages.clone(),
            // Different data everywhere data can differ, including the branches
            // a format takes: `project` says either a building or `none`, and
            // which one a sample caught is not a label.
            project: Some((Building::House, 3)),
            revel: "boring",
            revel_kind: "story",
            card: shown.card.as_ref().map(|card| CitizenCard {
                focus: Focus::BadlyDistracted,
                held: [false; THOUGHT_COUNT],
                hardship: Hardship::Broken,
                ..*card
            }),
            ..shown
        };
        other.mood = MOOD_MAX;
        let shared = words(&other);
        for word in words(&shown).into_iter().filter(|w| shared.contains(w)) {
            assert!(
                sample.contains(&word),
                "the page's sample never says `{word}`, so it is showing an older build"
            );
        }
    }
}
