//! Gate for the fg_refs tool: the fork's fg/bg table is derived
//! from the composed fixture the loader tests use, with the
//! deterministic primary-sprite convention (highest weight wins)
//! and -1 for absent slots — never invented indices.

#[test]
fn fg_refs_tool_emits_composed_indices() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../voxelizer/tests/composed/tile_config.json");
    let out = std::env::temp_dir().join(format!("fg_refs_{}.tsv", std::process::id()));
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_fg_refs"))
        .arg(&fixture)
        .arg(&out)
        .status()
        .expect("run fg_refs");
    assert!(status.success());
    let text = std::fs::read_to_string(&out).expect("read tsv");
    let row = |id: &str| -> (i64, i64) {
        let line = text
            .lines()
            .find(|l| l.starts_with(&format!("{id}\t")))
            .unwrap_or_else(|| panic!("no row for {id} in:\n{text}"));
        let mut it = line.split('\t');
        it.next();
        (
            it.next().unwrap().parse().unwrap(),
            it.next().unwrap().parse().unwrap(),
        )
    };
    assert_eq!(row("t_floor_wood"), (0, -1));
    assert_eq!(row("t_glass_window"), (1, -1));
    // Two ids sharing one entry share its sprite.
    assert_eq!(row("t_shared_a"), (3, -1));
    assert_eq!(row("t_shared_b"), (3, -1));
    // Weighted fg [1 @ w1, 3 @ w5]: the primary is the
    // highest-weight sprite (the voxelizer convention).
    assert_eq!(row("t_weighted"), (3, -1));
    let _ = std::fs::remove_file(&out);
}
