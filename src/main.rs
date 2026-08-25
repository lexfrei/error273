mod render;
mod sim;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

use sim::{Construction, Generator, Tick};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(80))))
        .init_resource::<Tick>()
        .init_resource::<Construction>()
        .insert_resource(Generator { fuel: 20 })
        .add_systems(Startup, (render::clear_screen, sim::setup).chain())
        .add_systems(
            Update,
            (
                sim::advance_tick,
                sim::construction,
                sim::citizen_ai,
                sim::colony_growth,
                sim::burn_fuel,
                render::render,
            )
                .chain(),
        )
        .run();
}
