//! S7 gate: the §5 scenario DSL. Every green scenario file in
//! `crates/sim/scenarios/` runs in-process against the world;
//! a subset also runs in process mode, driving a real
//! `sim-server` subprocess over its Unix socket with the same
//! assertions. The deferred files (T9, T12) must carry
//! `status = "s8"` and stay out of the green set — S8 owns
//! their PASS. The `capture` step must be refused at parse
//! time, naming the CaptureArgs limitation.
//!
//! Failing-first note: pre-S7 there was no scenario runner at
//! all (no `crates/sim/src/scenario.rs`, no `--scenario`
//! flag), so this gate fails against pre-S7 code by
//! construction — it does not compile there.

use std::path::PathBuf;

use sim::scenario::{run_in_process, run_process, ProcessConfig, Scenario};
use sim::{DefRegistry, Scene};

/// Scenarios green in-process (T1–T8 + T10-resync + T11).
const GREEN_IN_PROCESS: &[&str] = &[
    "t1_go_outside",
    "t2_open_ui",
    "t3_spawn_chase_kill",
    "t4_bash",
    "t5_door_cycle",
    "t6_time_weather",
    "t7_fields",
    "t8_items",
    "t10_resync",
    "t11_vehicles",
];

/// Scenarios green over a real socket (client-expressible
/// steps only; T3's twin authors its zombie into the scene).
const GREEN_PROCESS: &[&str] = &["t1_go_outside", "t3_process", "t5_door_cycle", "t10_resync"];

/// Written but not runnable until S8; must carry the status.
const DEFERRED_S8: &[&str] = &["t9_stairs", "t12_soak"];

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenarios")
}

fn scenes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scenes")
}

fn defs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/sim-defs/generated/defs.json")
}

fn load_scenario(stem: &str) -> Scenario {
    let text = std::fs::read_to_string(scenarios_dir().join(format!("{stem}.toml")))
        .unwrap_or_else(|e| panic!("read {stem}.toml: {e}"));
    Scenario::parse(&text).unwrap_or_else(|e| panic!("parse {stem}.toml: {e}"))
}

fn registry() -> DefRegistry {
    DefRegistry::load(&defs_path()).expect("registry loads")
}

/// The sim-server binary the dev-dependency builds. Cargo's
/// compile-time CARGO_BIN_EXE_<name> is not exposed for this
/// workspace's layout, so resolve at runtime: the env var when
/// present, else the binary beside the test profile dir
/// (`target/<profile>/deps/<test>` -> `target/<profile>/`).
fn server_bin() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_sim-server") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().expect("test exe path");
    let profile_dir = exe
        .parent()
        .and_then(|p| p.parent())
        .expect("profile dir above deps/");
    let bin = profile_dir.join("sim-server");
    assert!(
        bin.exists(),
        "sim-server binary not found at {} (build the workspace first)",
        bin.display()
    );
    bin
}

fn process_config() -> ProcessConfig {
    ProcessConfig {
        server_bin: server_bin(),
        defs_path: defs_path(),
        scenes_dir: scenes_dir(),
    }
}

#[test]
fn scenario_files_parse_and_carry_honest_status() {
    let mut seen: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(scenarios_dir()).expect("scenarios dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let stem = path
            .file_stem()
            .expect("file stem")
            .to_str()
            .expect("utf8 stem")
            .to_string();
        let sc = load_scenario(&stem);
        if DEFERRED_S8.contains(&stem.as_str()) {
            assert_eq!(
                sc.status.as_deref(),
                Some("s8"),
                "{stem} is deferred to S8 and must carry status = \"s8\""
            );
            assert!(
                !sc.is_green(),
                "{stem} must not present as green while deferred"
            );
        } else {
            assert!(
                sc.is_green(),
                "{stem} is in no green list and carries no s8 status — \
                 a scenario file must be one or the other"
            );
            assert!(
                GREEN_IN_PROCESS.contains(&stem.as_str()) || GREEN_PROCESS.contains(&stem.as_str()),
                "{stem} parses green but is listed in neither green set"
            );
        }
        seen.push(stem);
    }
    for stem in GREEN_IN_PROCESS
        .iter()
        .chain(GREEN_PROCESS.iter())
        .chain(DEFERRED_S8.iter())
    {
        assert!(
            seen.iter().any(|s| s == stem),
            "scenario file {stem}.toml is missing"
        );
    }
}

#[test]
fn capture_step_is_rejected_naming_the_limitation() {
    let text = "scene = \"street_block\"\nsteps = [ { do = \"capture\", name = \"x\" } ]\n";
    let err = Scenario::parse(text).expect_err("capture must be refused");
    assert!(
        err.contains("CaptureArgs"),
        "the refusal must name the CaptureArgs limitation, got: {err}"
    );
}

#[test]
fn world_event_steps_are_refused_in_process_mode() {
    // A spawn step over the wire has no honest client form;
    // the process runner must refuse it by name, not no-op it.
    let text = "scene = \"street_block\"\nsteps = [ { do = \"spawn\", type = \"mon_zombie\", pos = [12, 11, 0] } ]\n";
    let sc = Scenario::parse(text).expect("spawn parses (it is an in-process step)");
    assert!(sc.is_green());
    let reg = registry();
    let failure = run_process(&reg, &sc, &process_config())
        .expect_err("a world-event step must fail in process mode");
    assert!(
        failure.error.contains("world-event step"),
        "the refusal must name the reason, got: {}",
        failure.error
    );
}

#[test]
fn scenarios_green_in_process() {
    let reg = registry();
    for stem in GREEN_IN_PROCESS {
        let sc = load_scenario(stem);
        let scene = Scene::load_file(&scenes_dir().join(format!("{}.toml", sc.scene)))
            .unwrap_or_else(|e| panic!("{stem}: scene {}: {e}", sc.scene));
        run_in_process(&reg, scene, &sc)
            .unwrap_or_else(|f| panic!("{stem} (in-process) failed:\n{f}"));
    }
}

#[test]
fn scenarios_green_over_a_real_socket() {
    let reg = registry();
    let config = process_config();
    for stem in GREEN_PROCESS {
        let sc = load_scenario(stem);
        run_process(&reg, &sc, &config).unwrap_or_else(|f| panic!("{stem} (process) failed:\n{f}"));
    }
}
