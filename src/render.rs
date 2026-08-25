use bevy::prelude::*;

use crate::sim::{
    CENTER, Calendar, Citizen, Construction, Forest, Generator, HOUSE_WOOD_COST, House, Pos, R,
    Tick,
};

pub fn clear_screen() {
    print!("\x1B[2J");
}

pub fn render(
    tick: Res<Tick>,
    calendar: Res<Calendar>,
    generator: Res<Generator>,
    forest: Res<Forest>,
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
    for (cell, wood) in &forest.0 {
        grid[cell.y as usize][cell.x as usize] = if *wood > 0 { 'T' } else { 't' };
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
    let wood_left: u32 = forest.0.iter().map(|(_, w)| w).sum();
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
        "pop {:3}  houses {:3}  build {}  fuel {:4}  forest {:4}\n",
        alive,
        houses.iter().count(),
        site,
        generator.fuel,
        wood_left
    ));
    print!("{out}");

    if alive == 0 {
        println!("Everyone froze. The city is silent.");
        std::process::exit(0);
    }
}

fn citizen_glyph(citizen: &Citizen, pos: IVec2) -> char {
    if citizen.carrying {
        'W'
    } else if citizen.resting && pos == citizen.home {
        'z'
    } else {
        '@'
    }
}
