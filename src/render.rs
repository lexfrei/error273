use bevy::prelude::*;

use crate::sim::{CENTER, Citizen, Forest, Generator, Pos, R, Tick};

pub fn clear_screen() {
    print!("\x1B[2J");
}

pub fn render(
    tick: Res<Tick>,
    generator: Res<Generator>,
    forest: Res<Forest>,
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
    for (pos, citizen) in &citizens {
        grid[pos.0.y as usize][pos.0.x as usize] = if citizen.carrying { 'W' } else { '@' };
    }
    grid[CENTER.y as usize][CENTER.x as usize] = '#';

    let alive = citizens.iter().count();
    let wood_left: u32 = forest.0.iter().map(|(_, w)| w).sum();
    let mut out = String::from("\x1B[H");
    for row in &grid {
        out.push_str(&row.iter().collect::<String>());
        out.push('\n');
    }
    out.push_str(&format!(
        "tick {:5}  citizens {:2}  fuel {:3}  forest {:3}\n",
        tick.0, alive, generator.fuel, wood_left
    ));
    print!("{out}");

    if alive == 0 {
        println!("Everyone froze. The city is silent.");
        std::process::exit(0);
    }
}
