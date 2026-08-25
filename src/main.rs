#[cfg(not(feature = "window"))]
mod render;
mod sim;
mod status;
#[cfg(feature = "window")]
mod window;

#[cfg(not(feature = "window"))]
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
#[cfg(not(feature = "window"))]
use std::time::Duration;

/// The window steps the colony from the fixed schedule; headless steps it from
/// its own run loop, so both tick at the same rate.
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
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(headless_step())));
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
        .add_systems(Startup, sim::setup)
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
            )
                .chain(),
        );
    #[cfg(not(feature = "window"))]
    app.add_systems(SimSchedule, render::print_status.after(sim::burn_fuel));
    app.run();
}

/// How long the headless build waits between ticks. Watching wants the eighty
/// milliseconds; measuring a lifetime does not, and a citizen takes fifteen
/// game years to grow up, so `ERROR273_TURBO` drops the wait and runs as fast
/// as the machine will go. The status lines are the same either way.
#[cfg(not(feature = "window"))]
fn headless_step() -> Duration {
    if std::env::var_os("ERROR273_TURBO").is_some() {
        Duration::ZERO
    } else {
        Duration::from_millis(80)
    }
}
