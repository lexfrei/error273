mod render;
mod sim;
#[cfg(feature = "window")]
mod window;

#[cfg(not(feature = "window"))]
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
#[cfg(not(feature = "window"))]
use std::time::Duration;

/// Headless steps the colony from its own run loop; the window steps it from
/// the fixed 80 ms schedule, so both tick at the same rate.
#[cfg(feature = "window")]
use bevy::prelude::FixedUpdate as SimSchedule;
#[cfg(not(feature = "window"))]
use bevy::prelude::Update as SimSchedule;

use sim::{
    Air, Ballot, Built, Calendar, Construction, Generator, Granary, Lineage, Mayor, START_FOOD,
    START_FUEL, Tick,
};

fn main() {
    let mut app = App::new();
    #[cfg(not(feature = "window"))]
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(80))));
    #[cfg(feature = "window")]
    app.add_plugins(window::WindowRendererPlugin);
    app.init_resource::<Tick>()
        .init_resource::<Calendar>()
        .init_resource::<Construction>()
        .init_resource::<Built>()
        .init_resource::<Mayor>()
        .init_resource::<Ballot>()
        .init_resource::<Lineage>()
        .init_resource::<Air>()
        .insert_resource(Generator { fuel: START_FUEL })
        .insert_resource(Granary { food: START_FOOD })
        .add_systems(
            Startup,
            (render::clear_screen.run_if(terminal_draws), sim::setup).chain(),
        )
        .add_systems(
            SimSchedule,
            (
                sim::advance_tick,
                sim::advance_calendar,
                sim::aging,
                sim::count_buildings,
                sim::advance_weather,
                sim::regrow_patches,
                sim::construction,
                sim::citizen_ai,
                sim::colony_growth,
                sim::burn_fuel,
                render::render.run_if(terminal_draws),
            )
                .chain(),
        )
        .run();
}

/// The terminal renderer stands down when the window renderer owns the output.
fn terminal_draws() -> bool {
    !cfg!(feature = "window")
}
