//! Session recording (slice S8): `sim-server --record
//! <path>` writes the session as a §5 scenario file — the
//! scene name + the effective seed at the top, then one step
//! per recorded Input — so replaying the session IS running
//! the S7 scenario runner (in-process or against a fresh
//! server), and §3.1's determinism claim (same seed + same
//! inputs ⇒ byte-identical message log) becomes a gate
//! (record_gate.rs) instead of an assertion.
//!
//! What is recorded: every Input the scenario DSL can
//! express — pressed Keys, Text, MouseButton clicks, and
//! (via the server's Resize handling, see server.rs) — in
//! arrival order, across all connections the server serves
//! (the world persists across them, so the log is the whole
//! server's input history, in order). What is NOT recorded,
//! by design: Key releases, MouseMove, Focus and Wheel
//! inputs. Releases and MouseMove/Focus never change world
//! or UI state (rules.rs routing), so omitting them cannot
//! change a replay's message stream. Wheel inputs CAN move
//! a UI cursor in UiInput mode and the DSL has no wheel
//! step — a session that uses the wheel is therefore only
//! partially replayable; the recorder counts them and the
//! file's header comment says how many were dropped, so the
//! gap is on the record, never silent.
//!
//! The file is written when the server exits (Shutdown or
//! fatal handshake), atomically enough for a test tool: a
//! single `fs::write` of the assembled text.

use std::path::PathBuf;

use proto::{InputKind, InputT};

/// Accumulates scenario steps for one server run.
#[derive(Debug)]
pub struct Recorder {
    path: PathBuf,
    scene_name: String,
    seed: i64,
    steps: Vec<String>,
    dropped: usize,
}

fn toml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

/// The scenario-file spelling of a wire key code: the DSL's
/// `key_code` accepts a single character or "Esc".
fn key_step(code: i32) -> Option<String> {
    if code == ui_share::input::KEY_ESCAPE {
        return Some("{ do = \"key\", key = \"Esc\" }".to_string());
    }
    let c = char::from_u32(code as u32)?;
    if c.is_control() {
        return None; // not expressible in the DSL; counted as dropped
    }
    Some(format!(
        "{{ do = \"key\", key = \"{}\" }}",
        toml_escape(&c.to_string())
    ))
}

impl Recorder {
    pub fn new(path: PathBuf, scene_name: String, seed: i64) -> Recorder {
        Recorder {
            path,
            scene_name,
            seed,
            steps: Vec::new(),
            dropped: 0,
        }
    }

    /// Record one Input message's payload (the server calls
    /// this for every Input it receives, before applying it).
    pub fn record_input(&mut self, input: &InputT) {
        match input.kind {
            InputKind::Key => {
                if !input.pressed {
                    return; // releases never change state (module docs)
                }
                match key_step(input.key_code) {
                    Some(step) => self.steps.push(step),
                    None => self.dropped += 1,
                }
            }
            InputKind::Text => {
                if let Some(text) = &input.text {
                    self.steps.push(format!(
                        "{{ do = \"text\", text = \"{}\" }}",
                        toml_escape(text)
                    ));
                }
            }
            InputKind::MouseButton => {
                if input.pressed {
                    self.steps.push(format!(
                        "{{ do = \"click\", x = {}, y = {} }}",
                        input.mouse_x, input.mouse_y
                    ));
                }
            }
            InputKind::Wheel => {
                self.dropped += 1;
            }
            // MouseMove / Focus: no state, no step.
            _ => {}
        }
    }

    /// Record a Resize message (a step of its own in the DSL).
    pub fn record_resize(&mut self, width: u32, height: u32) {
        self.steps
            .push(format!("{{ do = \"resize\", width = {width}, height = {height} }}"));
    }

    /// How many inputs were received but not recorded
    /// (wheel + inexpressible keys — see the module docs).
    pub fn dropped(&self) -> usize {
        self.dropped
    }

    /// The scenario file's text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "# Recorded by sim-server --record (S8). Replay = run this file\n\
             # with the S7 scenario runner (`sim-server --scenario <file>`,\n\
             # in-process, or the runner's process mode against a fresh\n\
             # server): same scene + seed + inputs reproduce the session's\n\
             # message stream byte-for-byte (plan §3.1).\n",
        );
        if self.dropped > 0 {
            out.push_str(&format!(
                "# NOTE: {} input(s) were received but are not expressible\n\
                 # in the scenario DSL (wheel / control keys) and were NOT\n\
                 # recorded — this log replays the rest of the session.\n",
                self.dropped
            ));
        }
        out.push_str(&format!("scene = \"{}\"\n", toml_escape(&self.scene_name)));
        out.push_str(&format!("seed = {}\n", self.seed));
        out.push_str("name = \"recorded session\"\n\nsteps = [\n");
        for step in &self.steps {
            out.push_str("  ");
            out.push_str(step);
            out.push_str(",\n");
        }
        out.push_str("]\n");
        out
    }

    /// Write the file. Called once, at server exit.
    pub fn finish(&self) -> Result<(), String> {
        std::fs::write(&self.path, self.render())
            .map_err(|e| format!("record: cannot write {}: {e}", self.path.display()))
    }
}
