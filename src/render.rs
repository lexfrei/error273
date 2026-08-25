use bevy::prelude::*;

use crate::sim::{
    CENTER, Calendar, Cargo, Citizen, Construction, HOUSE_WOOD_COST, House, NeedKind, Pos, R,
    Stores, Tick,
};

pub fn clear_screen() {
    print!("\x1B[2J");
}

pub fn render(
    tick: Res<Tick>,
    calendar: Res<Calendar>,
    stores: Stores,
    construction: Res<Construction>,
    houses: Query<&Pos, With<House>>,
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
    for pos in &houses {
        grid[pos.0.y as usize][pos.0.x as usize] = 'H';
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
    let site = match &construction.site {
        Some(site) => format!("{:2}/{}", site.delivered, HOUSE_WOOD_COST),
        None => "  -  ".to_string(),
    };
    out.push_str(&format!(
        "tick {:5}  year {}  {:<6}  day {:2}  hour {:02}\n",
        tick.0,
        calendar.year,
        calendar.season.name(),
        calendar.day,
        calendar.hour
    ));
    out.push_str(&format!(
        "pop {:3}  houses {:3}  build {}  fuel {:4}  food {:4}  wood {:4}  game {:4}\n",
        alive,
        houses.iter().count(),
        site,
        stores.generator.fuel,
        stores.granary.food,
        standing(Cargo::Wood),
        standing(Cargo::Food)
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
