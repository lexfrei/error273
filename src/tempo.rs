//! How often a tick happens in real time, and nothing about what a tick means.
//!
//! One tick is one game hour whatever the tempo: this module moves the wall
//! clock only, never the rate constants the simulation is written in.

use std::time::Duration;

pub const TICK_MS_DEFAULT: u64 = 80;
/// A sane range for the knob. Below the floor the colony outruns the terminal
/// it is being watched in; above the ceiling a season takes a working day.
pub const TICK_MS_MIN: u64 = 5;
pub const TICK_MS_MAX: u64 = 5_000;
// The default has to sit inside its own range, checked where it cannot be
// skipped rather than left to a test that might be deleted.
const _: () = assert!(TICK_MS_MIN <= TICK_MS_DEFAULT && TICK_MS_DEFAULT <= TICK_MS_MAX);
/// The most ticks a single wake-up will run back to back to regain the grid.
/// Without it a machine that stalls, or a laptop that sleeps, would come back
/// and run hours of colony in one frozen frame.
#[cfg(not(feature = "window"))]
pub const MAX_CATCHUP_TICKS: u64 = 64;

pub const TICK_MS_ENV: &str = "ERROR273_TICK_MS";
/// Turbo is an instrument, not a way to watch, so only the headless build has
/// it: a window drawing frames as fast as the CPU allows shows nothing.
#[cfg(not(feature = "window"))]
pub const TURBO_ENV: &str = "ERROR273_TURBO";

pub fn clamp_tick_ms(ms: u64) -> u64 {
    ms.clamp(TICK_MS_MIN, TICK_MS_MAX)
}

/// The tick length asked for at launch, clamped. Anything unparseable is the
/// default rather than an error: this is a tempo knob, not a configuration.
pub fn tick_step() -> Duration {
    let ms = std::env::var(TICK_MS_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .map_or(TICK_MS_DEFAULT, clamp_tick_ms);
    Duration::from_millis(ms)
}

#[cfg(not(feature = "window"))]
pub fn turbo() -> bool {
    std::env::var_os(TURBO_ENV).is_some()
}

/// How long after the founding instant tick `n` is due. The grid is absolute --
/// tick `n` is at `n * step`, never at "the last tick plus step" -- so a slow
/// frame or a long sleep cannot push the whole run late behind it.
#[cfg(not(feature = "window"))]
pub fn tick_due_nanos(step: Duration, n: u64) -> u64 {
    (step.as_nanos() as u64).saturating_mul(n)
}

/// How many ticks to run right now: everything the grid says is due and has not
/// been run, capped so a long stall is caught up over several wake-ups rather
/// than one.
#[cfg(not(feature = "window"))]
pub fn ticks_due(elapsed_nanos: u64, step: Duration, already_run: u64) -> u64 {
    let step_nanos = (step.as_nanos() as u64).max(1);
    let owed = (elapsed_nanos / step_nanos).saturating_sub(already_run);
    owed.min(MAX_CATCHUP_TICKS)
}

/// How long to wait before tick `n` comes due, given where the clock is now.
#[cfg(not(feature = "window"))]
pub fn wait_until(step: Duration, n: u64, elapsed_nanos: u64) -> Duration {
    Duration::from_nanos(tick_due_nanos(step, n).saturating_sub(elapsed_nanos))
}

#[cfg(all(test, not(feature = "window")))]
mod grid_tests {
    use super::*;

    const STEP: Duration = Duration::from_millis(80);

    #[test]
    fn the_grid_is_the_founding_instant_times_the_step() {
        assert_eq!(tick_due_nanos(STEP, 0), 0);
        assert_eq!(tick_due_nanos(STEP, 1), 80_000_000);
        assert_eq!(tick_due_nanos(STEP, 12), 960_000_000);
    }

    #[test]
    fn the_grid_does_not_overflow_on_a_long_run() {
        let far = tick_due_nanos(STEP, u64::MAX);
        assert!(far > 0, "a saturating grid is better than a wrapping one");
    }

    #[test]
    fn nothing_is_due_before_the_first_step_has_passed() {
        assert_eq!(ticks_due(0, STEP, 0), 0);
        assert_eq!(ticks_due(79_999_999, STEP, 0), 0);
        assert_eq!(ticks_due(80_000_000, STEP, 0), 1);
    }

    #[test]
    fn what_has_already_run_is_not_owed_again() {
        let three_and_a_half = tick_due_nanos(STEP, 3) + 40_000_000;
        assert_eq!(ticks_due(three_and_a_half, STEP, 0), 3);
        assert_eq!(ticks_due(three_and_a_half, STEP, 3), 0);
        assert_eq!(ticks_due(three_and_a_half, STEP, 2), 1);
    }

    #[test]
    fn a_long_stall_is_caught_up_over_several_wakings() {
        let an_age = tick_due_nanos(STEP, MAX_CATCHUP_TICKS * 10);
        assert_eq!(
            ticks_due(an_age, STEP, 0),
            MAX_CATCHUP_TICKS,
            "a laptop coming out of sleep must not run an hour of colony in one frame"
        );
    }

    #[test]
    fn a_sleep_that_overshoots_does_not_move_the_grid() {
        // Every wake-up lands a little late, the way a real sleep does.
        let mut clock = 0u64;
        let mut ran = 0u64;
        let overshoot = 3_000_000;
        for _ in 0..500 {
            ran += ticks_due(clock, STEP, ran);
            clock += wait_until(STEP, ran + 1, clock).as_nanos() as u64 + overshoot;
        }
        let owed = clock / tick_due_nanos(STEP, 1);
        assert!(
            owed.saturating_sub(ran) <= 1,
            "after 500 late wake-ups the run is {} ticks behind the wall",
            owed - ran
        );
    }

    #[test]
    fn a_frame_that_runs_long_does_not_move_the_grid_either() {
        let mut clock = 0u64;
        let mut ran = 0u64;
        for frame in 0..500 {
            // Every tenth frame takes three steps to compute.
            let compute = if frame % 10 == 0 {
                240_000_000
            } else {
                1_000_000
            };
            ran += ticks_due(clock, STEP, ran);
            clock += compute;
            clock += wait_until(STEP, ran + 1, clock).as_nanos() as u64;
        }
        let owed = clock / tick_due_nanos(STEP, 1);
        assert!(
            owed.saturating_sub(ran) <= 1,
            "the grid slipped by {} ticks",
            owed - ran
        );
    }
}

#[cfg(test)]
mod knob_tests {
    use super::*;

    #[test]
    fn the_knob_is_held_inside_a_sane_range() {
        assert_eq!(clamp_tick_ms(TICK_MS_DEFAULT), TICK_MS_DEFAULT);
        assert_eq!(clamp_tick_ms(0), TICK_MS_MIN);
        assert_eq!(clamp_tick_ms(u64::MAX), TICK_MS_MAX);
        for ms in [TICK_MS_MIN, TICK_MS_DEFAULT, TICK_MS_MAX] {
            assert_eq!(
                clamp_tick_ms(ms),
                ms,
                "{ms} is inside the range and must pass"
            );
        }
    }
}
