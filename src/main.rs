mod render;
mod sim;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

use sim::{
    Ballot, Built, Calendar, Construction, Generator, Granary, Lineage, Mayor, START_FOOD,
    START_FUEL, Tick,
};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(80))))
        .init_resource::<Tick>()
        .init_resource::<Calendar>()
        .init_resource::<Construction>()
        .init_resource::<Built>()
        .init_resource::<Mayor>()
        .init_resource::<Ballot>()
        .init_resource::<Lineage>()
        .insert_resource(Generator { fuel: START_FUEL })
        .insert_resource(Granary { food: START_FOOD })
        .add_systems(Startup, (render::clear_screen, sim::setup).chain())
        .add_systems(
            Update,
            (
                sim::advance_tick,
                sim::advance_calendar,
                sim::aging,
                sim::count_buildings,
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
