//! The headless face of the game: no map, just the status lines on stdout every
//! tick. It is what the balance log and the gate runs read, so its output is an
//! instrument rather than a display.

use bevy::prelude::*;

use crate::sim::{
    BUILDINGS, Ballot, Cargo, Citizen, Construction, Outside, Regard, STAT_COUNT, STATS, Stores,
    Structure, estimate, is_known, median, regard_of,
};
use crate::status::{CitizenCard, Status, status_lines};

pub fn print_status(
    outside: Outside,
    stores: Stores,
    construction: Res<Construction>,
    ballot: Res<Ballot>,
    structures: Query<&Structure>,
    citizens: Query<&Citizen>,
) {
    let standing = |kind: Cargo| -> u32 {
        stores
            .patches
            .0
            .iter()
            .filter(|patch| patch.kind == kind)
            .map(|patch| patch.amount)
            .sum()
    };
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
                seed: citizen.seed,
                age: citizen.age,
                words,
                watched: citizen.watched,
                known: is_known(citizen.watched),
            }
        });
    let status = Status {
        tick: outside.tick.0,
        calendar: *outside.calendar,
        ambient: outside.air.ambient,
        alive: ages.len(),
        fuel: stores.generator.fuel,
        food: stores.granary.food,
        wood: standing(Cargo::Wood),
        game: standing(Cargo::Food),
        buildings,
        project: construction
            .site
            .as_ref()
            .map(|site| (site.building, site.delivered)),
        tally: ballot.tally,
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
