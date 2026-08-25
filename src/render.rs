use bevy::prelude::*;

use crate::sim::{
    ADULT_AGE, BUILDINGS, Ballot, Building, CENTER, Cargo, Citizen, Construction, FRAILTY_ONSET,
    NeedKind, Outside, Pos, R, Stores, Structure, Tick, couples, is_adult,
};

pub fn clear_screen() {
    print!("\x1B[2J");
}

pub fn render(
    tick: Res<Tick>,
    outside: Outside,
    stores: Stores,
    construction: Res<Construction>,
    ballot: Res<Ballot>,
    structures: Query<(&Pos, &Structure)>,
    citizens: Query<(&Pos, &Citizen)>,
) {
    let size = (R * 2 + 1) as usize;
    let mut grid = vec![vec![' '; size]; size];
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let p = IVec2::new(x as i32, y as i32);
            if p.as_vec2().distance(CENTER.as_vec2()) <= R as f32 + 0.5 {
                *cell = '.';
            }
        }
    }
    for patch in &stores.patches.0 {
        grid[patch.pos.y as usize][patch.pos.x as usize] = patch_glyph(patch.kind, patch.amount);
    }
    for (pos, structure) in &structures {
        grid[pos.0.y as usize][pos.0.x as usize] = structure.0.rules().glyph;
    }
    if let Some(site) = &construction.site {
        grid[site.pos.y as usize][site.pos.x as usize] = '+';
    }
    for (pos, citizen) in &citizens {
        grid[pos.0.y as usize][pos.0.x as usize] = citizen_glyph(citizen, pos.0);
    }
    grid[CENTER.y as usize][CENTER.x as usize] = '#';

    let alive = citizens.iter().count();
    let standing = |kind: Cargo| -> u32 {
        stores
            .patches
            .0
            .iter()
            .filter(|patch| patch.kind == kind)
            .map(|patch| patch.amount)
            .sum()
    };
    let mut out = String::from("\x1B[H");
    for row in &grid {
        out.push_str(&row.iter().collect::<String>());
        out.push('\n');
    }
    let standing_count = |building: Building| {
        structures
            .iter()
            .filter(|(_, structure)| structure.0 == building)
            .count()
    };
    let project = match &construction.site {
        Some(site) => format!(
            "{} {}/{}",
            site.building.rules().name,
            site.delivered,
            site.building.rules().cost
        ),
        None => "none".to_string(),
    };
    out.push_str(&format!(
        "tick {:5}  year {}  {:<6}  day {:2}  hour {:02}  air {:+.0}\n",
        tick.0,
        outside.calendar.year,
        outside.calendar.season.name(),
        outside.calendar.day,
        outside.calendar.hour,
        outside.air.ambient
    ));
    out.push_str(&format!(
        "pop {:3}  fuel {:4}  food {:4}  wood {:4}  game {:4}\n",
        alive,
        stores.generator.fuel,
        stores.granary.food,
        standing(Cargo::Wood),
        standing(Cargo::Food)
    ));
    let counts: Vec<String> = BUILDINGS
        .into_iter()
        .map(|building| format!("{} {:3}", building.rules().name, standing_count(building)))
        .collect();
    let votes: Vec<String> = BUILDINGS
        .into_iter()
        .map(|building| format!("{:.0}", ballot.tally[building as usize]))
        .collect();
    out.push_str(&format!(
        "{}  project {}  vote {}\n",
        counts.join("  "),
        project,
        votes.join("/")
    ));
    let ages: Vec<f32> = citizens.iter().map(|(_, citizen)| citizen.age).collect();
    let children = ages.iter().filter(|age| !is_adult(**age)).count();
    let frail = ages.iter().filter(|age| **age > FRAILTY_ONSET).count();
    out.push_str(&format!(
        "under {:<2} {:3}  grown {:3}  over {:<2} {:3}  couples {:3}\n",
        ADULT_AGE as u32,
        children,
        alive - children - frail,
        FRAILTY_ONSET as u32,
        frail,
        couples(&ages)
    ));
    print!("{out}");

    if alive == 0 {
        println!("The colony is silent.");
        std::process::exit(0);
    }
}

fn patch_glyph(kind: Cargo, amount: u32) -> char {
    match (kind, amount > 0) {
        (Cargo::Wood, true) => 'T',
        (Cargo::Wood, false) => 't',
        (Cargo::Food, true) => 'Y',
        (Cargo::Food, false) => 'y',
    }
}

fn citizen_glyph(citizen: &Citizen, pos: IVec2) -> char {
    match citizen.carrying {
        Some(Cargo::Wood) => 'W',
        Some(Cargo::Food) => 'F',
        None if citizen.needs.get(NeedKind::Rest).pressing && pos == citizen.home => 'z',
        None => '@',
    }
}
