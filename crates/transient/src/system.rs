//! The transient emitter lifecycle: ingest → active → expired.
//!
//! Time is an explicit millisecond clock the caller advances
//! (engine frame time in the live loop; scripted values in the
//! gates). Nothing here reads a wall clock, so every gate is
//! deterministic.

use std::collections::HashSet;

use proto::{EffectEventT, EffectKind, EffectsT};

use crate::params::{params_for, EmitterParams};

/// Identity of one spawned emitter: the `Effects` message's seq
/// plus the event's index within it. A re-sent message (the
/// protocol's resync paths can produce one) re-presents the same
/// keys and spawns nothing twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmitterKey {
    pub seq: u64,
    pub index: u32,
}

/// One live (or not-yet-expired) transient effect.
#[derive(Debug, Clone, PartialEq)]
pub struct Emitter {
    pub key: EmitterKey,
    pub kind: EffectKind,
    pub effect_id: String,
    /// Game tile coordinates as floats (x east, y south, z up).
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub start_ms: u64,
    pub duration_ms: u32,
    /// `None` for sound markers (no visual emitter; the record
    /// exists so the engine can route the marker to audio).
    pub params: Option<EmitterParams>,
}

impl Emitter {
    /// Progress in [0, 1] at `now_ms`. A zero-duration event is an
    /// instant marker: progress is 1 from its start instant on.
    pub fn progress(&self, now_ms: u64) -> f32 {
        if self.duration_ms == 0 {
            return if now_ms >= self.start_ms { 1.0 } else { 0.0 };
        }
        let elapsed = now_ms.saturating_sub(self.start_ms) as f32;
        (elapsed / self.duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Alive at `now_ms`: started, and not yet at the end of its
    /// duration. The end instant itself is expired — an effect
    /// with duration D spawned at T occupies [T, T+D), so at T+D
    /// the next frame is clean (gate 8.5: no leftovers).
    pub fn alive(&self, now_ms: u64) -> bool {
        now_ms >= self.start_ms && now_ms < self.start_ms + self.duration_ms as u64
    }

    /// Current position in game tile space. Projectiles travel
    /// from → to linearly over the duration (the protocol carries
    /// endpoints + duration and no curve — see crate docs); every
    /// other kind sits at its target tile (`to`), which for a
    /// burst/flash is the tile the game says it happens on.
    pub fn position(&self, now_ms: u64) -> [f32; 3] {
        if self.kind != EffectKind::Projectile {
            return self.to;
        }
        let t = self.progress(now_ms);
        [
            self.from[0] + (self.to[0] - self.from[0]) * t,
            self.from[1] + (self.to[1] - self.from[1]) * t,
            self.from[2] + (self.to[2] - self.from[2]) * t,
        ]
    }
}

fn pos(p: &Option<proto::Vec3iT>) -> [f32; 3] {
    match p {
        Some(v) => [v.x as f32, v.y as f32, v.z as f32],
        None => [0.0, 0.0, 0.0],
    }
}

#[derive(Debug, Default)]
pub struct TransientSystem {
    emitters: Vec<Emitter>,
    spawned: HashSet<EmitterKey>,
    now_ms: u64,
}

impl TransientSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn now_ms(&self) -> u64 {
        self.now_ms
    }

    /// Ingest one `Effects` message at the current clock. Returns
    /// how many emitters were newly spawned. Events whose key was
    /// already spawned (re-send) are skipped; a message whose seq
    /// is older than one already ingested is dropped whole — its
    /// effects belong to a past the renderer has already shown,
    /// and replaying them would put effects on the wrong tiles at
    /// the wrong times (gate 8.5).
    pub fn ingest(&mut self, effects: &EffectsT) -> usize {
        if let Some(max) = self.spawned.iter().map(|k| k.seq).max() {
            if effects.seq < max {
                return 0;
            }
        }
        let mut spawned_now = 0;
        if let Some(events) = &effects.events {
            for (i, ev) in events.iter().enumerate() {
                let key = EmitterKey {
                    seq: effects.seq,
                    index: i as u32,
                };
                if !self.spawned.insert(key) {
                    continue;
                }
                self.emitters
                    .push(Emitter::from_event(key, ev, self.now_ms));
                spawned_now += 1;
            }
        }
        spawned_now
    }

    /// Advance the clock and expire finished emitters. The clock
    /// is monotonic: moving it backwards is ignored (a stale frame
    /// timestamp must not resurrect expired effects).
    pub fn advance(&mut self, now_ms: u64) {
        if now_ms > self.now_ms {
            self.now_ms = now_ms;
        }
        let now = self.now_ms;
        self.emitters.retain(|e| e.alive(now));
    }

    /// Emitters alive at the current clock, in spawn order.
    pub fn active(&self) -> &[Emitter] {
        &self.emitters
    }

    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }
}

impl Emitter {
    fn from_event(key: EmitterKey, ev: &EffectEventT, start_ms: u64) -> Self {
        Self {
            key,
            kind: ev.kind,
            effect_id: ev.effect_id.clone().unwrap_or_default(),
            from: pos(&ev.from_pos),
            to: pos(&ev.to_pos),
            start_ms,
            duration_ms: ev.duration_ms,
            params: params_for(ev.kind, ev.effect_id.as_deref().unwrap_or("")),
        }
    }
}
