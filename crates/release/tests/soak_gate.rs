//! Gates 8.8/8.9: the 2 h scripted soak, alone in its test binary
//! so the process RSS it samples belongs to the soak itself.
//!
//! Assertions are the release criteria at this level: the session
//! completes, transient state drains to empty, the vehicle store
//! holds only live vehicles, per-step cost does not creep, and
//! resident memory after warm-up grows by less than 5% + 2 MiB of
//! allocator slack over the remaining ~1.8 h.

use release::{run_soak, SoakConfig};

#[test]
fn gate_8_8_8_9_two_hour_scripted_soak() {
    let stats = run_soak(SoakConfig { steps: 7200 });
    println!(
        "soak: steps={} max_transient={} end_transient={} vehicles={} toggles={} \
         rss start/warm/end={:?}/{:?}/{:?} KiB step cost first/last={:.1}/{:.1} us",
        stats.steps,
        stats.max_transient_alive,
        stats.transient_alive_at_end,
        stats.vehicles_at_end,
        stats.door_toggles,
        stats.rss_start_kib,
        stats.rss_warm_kib,
        stats.rss_end_kib,
        stats.first_block_us_per_step,
        stats.last_block_us_per_step,
    );
    assert_eq!(stats.steps, 7200);
    assert!(
        stats.max_transient_alive > 0,
        "effects actually fired during the soak"
    );
    assert!(
        stats.max_transient_alive <= 4,
        "emitters stay bounded (2 per burst, bursts overlap < 2 deep)"
    );
    assert_eq!(
        stats.transient_alive_at_end, 0,
        "no leftover effects at session end"
    );
    assert_eq!(
        stats.vehicles_at_end, 2,
        "exactly the live set remains: the driving car + the traffic vehicle, \
         which is inside its present window at step 7199 (7199 % 500 = 199 < 250)"
    );
    assert!(stats.door_toggles > 600, "the door toggled all session");
    // No cost creep: the last 600 steps cost within 3x the first
    // 600 (generous bound for a shared CI VM; the recorded numbers
    // are the regression baseline).
    assert!(
        stats.last_block_us_per_step < stats.first_block_us_per_step * 3.0 + 50.0,
        "per-step cost crept: first {:.1} us, last {:.1} us",
        stats.first_block_us_per_step,
        stats.last_block_us_per_step
    );
    if let (Some(warm), Some(end)) = (stats.rss_warm_kib, stats.rss_end_kib) {
        let growth = end as i64 - warm as i64;
        assert!(
            growth < (warm as i64 / 20) + 2048,
            "RSS grew {growth} KiB after warm-up (warm {warm}, end {end}) — leak suspect"
        );
    }
}
