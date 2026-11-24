//! Generate the fork exporter's fg/bg sprite-ref table from a
//! tileset's COMPOSED `tile_config.json`.
//!
//! The fork's TileDefs carry `fg_ref` / `bg_ref`: indices into
//! the composed config the game's tiles renderer resolves
//! sprites from. The exporter cannot invent those indices
//! (docs/phase3-fork.md §4.4) — this tool derives them from the
//! real composed config (produced by the game's
//! `tools/gfx_tools/compose.py` or a tileset release) and writes
//! the TSV the fork adapter loads at startup
//! (`CDDA_VOXEL_FG_REFS`, parsed by `voxel_ipc::FgBgRefs`):
//!
//! ```text
//! # string_id <TAB> fg_ref <TAB> bg_ref
//! t_floor_wood	12	3
//! ```
//!
//! Per entry, the ref is the entry's *primary* sprite
//! (`SpriteRef::primary`: highest weight, ties by list order —
//! the voxelizer's deterministic-choice convention). An entry
//! with no fg (or bg) gets -1 for that slot. Refs are the
//! section-local composed indices, matching the schema's
//! single-int fields; a tileset whose sections are separate
//! sheets shares the index space the same way the game's own
//! composed loader presents it per section (recorded limit of
//! the single-int wire field, not something this tool papers
//! over).
//!
//! Usage: `fg_refs <tile_config.json> <out.tsv>`

use std::process::ExitCode;

fn primary_index(r: &Option<tileset::SpriteRef>) -> i64 {
    match r.as_ref().and_then(|s| s.primary()) {
        Some(tileset::SpriteRef::Index(i)) => i as i64,
        _ => -1,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: fg_refs <tile_config.json> <out.tsv>");
        return ExitCode::from(2);
    }
    let ts = match tileset::load_composed(std::path::Path::new(&args[1])) {
        Ok(ts) => ts,
        Err(e) => {
            eprintln!("fg_refs: cannot load {}: {e}", args[1]);
            return ExitCode::FAILURE;
        }
    };
    if !ts.composed {
        eprintln!("fg_refs: {} did not load as a composed tileset", args[1]);
        return ExitCode::FAILURE;
    }
    let mut out = String::from(
        "# fg/bg sprite refs from composed tile_config.json\n\
         # string_id\tfg_ref\tbg_ref\n",
    );
    let mut with_fg = 0usize;
    for (id, entry) in &ts.entries {
        let fg = primary_index(&entry.fg);
        let bg = primary_index(&entry.bg);
        if fg >= 0 {
            with_fg += 1;
        }
        out.push_str(&format!("{id}\t{fg}\t{bg}\n"));
    }
    if let Err(e) = std::fs::write(&args[2], out) {
        eprintln!("fg_refs: cannot write {}: {e}", args[2]);
        return ExitCode::FAILURE;
    }
    println!(
        "fg_refs: {} entries ({} with an fg ref) -> {}",
        ts.entries.len(),
        with_fg,
        args[2]
    );
    ExitCode::SUCCESS
}
