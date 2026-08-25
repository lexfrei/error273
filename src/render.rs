//! The headless face of the game: no map, just the status lines on stdout every
//! tick. It is what the balance log and the gate runs read, so its output is an
//! instrument rather than a display.

use bevy::prelude::*;

use crate::sim::{BUILDINGS, Ballot, Cargo, Citizen, Construction, Outside, Stores, Structure};
use crate::status::{Status, status_lines};

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
    };
    for line in status_lines(&status) {
        println!("{line}");
    }

    if status.alive == 0 {
        println!("The colony is silent.");
        std::process::exit(0);
    }
}
