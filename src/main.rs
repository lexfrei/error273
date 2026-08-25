#[cfg(not(feature = "window"))]
mod render;
mod sim;
mod status;
mod tempo;
#[cfg(feature = "window")]
mod window;

use bevy::prelude::*;

/// The window steps the colony from the fixed schedule; headless steps it from
/// its own run loop, so both tick at the same rate.
#[cfg(feature = "window")]
use bevy::prelude::FixedUpdate as SimSchedule;
#[cfg(not(feature = "window"))]
use bevy::prelude::Update as SimSchedule;

use sim::{
    Air, Ballot, Built, Calendar, Construction, Flow, Generator, Granary, Lineage, Mayor,
    START_FOOD, START_FUEL, Tick, Trend,
};

fn main() {
    let mut app = App::new();
    #[cfg(not(feature = "window"))]
    {
        app.add_plugins(MinimalPlugins);
        app.set_runner(run_on_the_grid);
    }
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
        .init_resource::<Trend>()
        .init_resource::<Flow>()
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
                sim::record_trend,
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

/// The headless run loop, holding an absolute grid: tick `n` is due at `n`
/// steps after the colony was founded, not a step after whatever the last tick
/// happened to finish. Bevy's own loop subtracts the frame's compute time from
/// the wait, which is most of the way there, but the wait itself overshoots and
/// nothing ever takes that back, so a long run drifts by the sum of its own
/// sleeps. Recomputing what is due from the founding instant cannot drift.
///
/// `ERROR273_TURBO` drops the grid entirely and runs flat out, because
/// measuring a fifteen-year childhood is not watching.
#[cfg(not(feature = "window"))]
fn run_on_the_grid(mut app: App) -> AppExit {
    use bevy::app::PluginsState;
    use std::time::Instant;

    if app.plugins_state() != PluginsState::Cleaned {
        while app.plugins_state() == PluginsState::Adding {
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }

    let turbo = tempo::turbo();
    let step = tempo::tick_step();
    let founded = Instant::now();
    let mut ran: u64 = 0;
    loop {
        if turbo {
            app.update();
            if let Some(exit) = app.should_exit() {
                return exit;
            }
            continue;
        }
        let due = tempo::ticks_due(founded.elapsed().as_nanos() as u64, step, ran);
        for _ in 0..due {
            app.update();
            if let Some(exit) = app.should_exit() {
                return exit;
            }
            ran += 1;
        }
        let wait = tempo::wait_until(step, ran + 1, founded.elapsed().as_nanos() as u64);
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
    }
}
