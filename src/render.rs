//! The headless face of the game: no map, just the status lines on stdout every
//! tick. It is what the balance log and the gate runs read, so its output is an
//! instrument rather than a display.

use bevy::prelude::*;

use crate::sim::{
    BUILDINGS, Cargo, Citizen, Patches, Pos, Regard, STAT_COUNT, STATS, Structure, estimate,
    focus_band, focus_of, is_known, median, on_frame, regard_of, world_is_bounded,
};
use crate::status::{CitizenCard, Readings, Status, status_lines};

/// The memory ceiling, checked where the gates can see it fail. This is the
/// headless build's business and not the simulation's -- see `world_is_bounded`.
pub fn hold_the_world_bound(patches: Res<Patches>) {
    assert!(
        world_is_bounded(patches.held_cells()),
        "the colony is holding {} cells of world, past the ceiling",
        patches.held_cells()
    );
}

pub fn print_status(
    readings: Readings,
    structures: Query<&Structure>,
    citizens: Query<&Citizen>,
    walkers: Query<&Pos, With<Citizen>>,
) {
    let mut buildings = [0usize; BUILDINGS.len()];
    for structure in &structures {
        buildings[structure.0 as usize] += 1;
    }
    let ages: Vec<f32> = citizens.iter().map(|citizen| citizen.age).collect();
    let mut stats = [0.0; STAT_COUNT];
    for stat in STATS {
        let mut held: Vec<f32> = citizens
            .iter()
            .map(|citizen| citizen.upbringing.stats().of(stat))
            .collect();
        stats[stat as usize] = median(&mut held);
    }
    let card = citizens
        .iter()
        .max_by(|a, b| a.age.total_cmp(&b.age))
        .map(|citizen| {
            let mut words = [Regard::Middling; STAT_COUNT];
            for stat in STATS {
                let guess = estimate(
                    citizen.upbringing.stats().of(stat),
                    citizen.upbringing.prosperity(),
                    citizen.watched,
                );
                words[stat as usize] = regard_of(guess, stats[stat as usize]);
            }
            CitizenCard {
                focus: focus_band(focus_of(&citizen.needs)),
                seed: citizen.seed,
                age: citizen.age,
                words,
                watched: citizen.watched,
                known: is_known(citizen.watched),
            }
        });
    let status = Status {
        tick: readings.outside.tick.0,
        calendar: *readings.outside.calendar,
        ambient: readings.outside.air.ambient,
        alive: ages.len(),
        off_frame: walkers.iter().filter(|pos| !on_frame(pos.0)).count(),
        missing: readings.missing.count(),
        fuel: readings.stores.generator.fuel,
        food: readings.stores.granary.food,
        wood: readings.standing(Cargo::Wood),
        game: readings.standing(Cargo::Food),
        buildings,
        project: readings
            .construction
            .site
            .as_ref()
            .map(|site| (site.building, site.delivered)),
        tally: readings.ballot.tally,
        ages,
        stats,
        card,
    };
    for line in status_lines(&status) {
        println!("{line}");
    }

    if status.alive == 0 {
        println!("The colony is silent.");
        std::process::exit(0);
    }
}
