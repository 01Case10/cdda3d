//! Runs flatc on schema/cdda_ipc.fbs to generate the Rust bindings into
//! OUT_DIR (cdda_ipc_generated.rs), plus a "v2" variant with one extra field
//! on Hello (cdda_ipc_v2_generated.rs) that the forward-compatibility test
//! (gate 1.9) uses to prove old readers ignore unknown fields, plus a "v1"
//! variant generated from the frozen pre-Phase-10 schema fixture
//! (tests/fixtures/cdda_ipc_v1.fbs) that the old-reader test (gate 10.1a)
//! uses to prove the pre-10.1 reader ignores EnvironmentUpdate.
//!
//! flatc 24.3.25 is located via, in order: $FLATC, the project-local copy at
//! <repo>/tools/flatbuffers/flatc (i.e. ~/workspace/tools/flatbuffers/flatc
//! when the repo lives in ~/workspace/cdda-voxel — see find_flatc), /tmp/flatc,
//! then $PATH. A missing flatc is a hard build error with instructions, never
//! a silent fallback to stale generated code.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_flatc(manifest_dir: &Path) -> PathBuf {
    if let Ok(f) = env::var("FLATC") {
        if Path::new(&f).exists() {
            return PathBuf::from(f);
        }
    }
    // Repo-local tool dir: <repo>/../../tools relative to the workspace root.
    // crates/proto -> workspace root is ../..; the VM keeps tools at
    // ~/workspace/tools/flatbuffers/flatc, i.e. two levels above the root.
    let root = manifest_dir.join("../..");
    let candidates = [
        root.join("tools/flatbuffers/flatc"),
        root.join("../tools/flatbuffers/flatc"),
        PathBuf::from("/tmp/flatc"),
        PathBuf::from("/home/hatch/workspace/tools/flatbuffers/flatc"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("flatc") // last resort: PATH lookup, error below explains.
}

fn run_flatc(flatc: &Path, schema: &Path, out_dir: &Path) {
    let status = Command::new(flatc)
        .args(["--rust", "--gen-object-api", "-o"])
        .arg(out_dir)
        .arg(schema)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => panic!(
            "flatc failed ({s}) on {}. Is flatc 24.3.25 installed? \
             Set FLATC=/path/to/flatc (project copy: ~/workspace/tools/flatbuffers/flatc).",
            schema.display()
        ),
        Err(e) => panic!(
            "could not run flatc at {}: {e}. Install flatbuffers 24.3.25 or set \
             FLATC=/path/to/flatc (project copy: ~/workspace/tools/flatbuffers/flatc).",
            flatc.display()
        ),
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let schema = manifest_dir.join("../../schema/cdda_ipc.fbs");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed={}", schema.display());
    println!("cargo:rerun-if-changed=build.rs");

    let flatc = find_flatc(&manifest_dir);
    run_flatc(&flatc, &schema, &out_dir);

    // v2 schema for the forward-compat test: Hello gains one field. Insert it
    // just before the closing brace of the Hello table.
    // Normalise line endings before the marker surgery below: a
    // Windows checkout with core.autocrlf=true gives the schema CRLF
    // endings and the LF marker would never match (hit on Joey's PC,
    // 2026-11-24: "schema changed: Hello/z_max marker gone").
    let src = fs::read_to_string(&schema)
        .expect("read schema")
        .replace("\r\n", "\n");
    let marker = "  z_max: int32;\n}";
    assert!(
        src.contains(marker),
        "schema changed: Hello/z_max marker gone"
    );
    let v2 = src.replacen(marker, "  z_max: int32;\n  future_field: string;\n}", 1);
    let v2_schema = out_dir.join("cdda_ipc_v2.fbs");
    fs::write(&v2_schema, v2).expect("write v2 schema");
    let v2_out = out_dir.join("v2");
    fs::create_dir_all(&v2_out).unwrap();
    run_flatc(&flatc, &v2_schema, &v2_out);
    fs::rename(
        v2_out.join("cdda_ipc_v2_generated.rs"),
        out_dir.join("cdda_ipc_v2_generated.rs"),
    )
    .expect("rename v2 generated file");

    // v1 schema for the Phase 10 old-reader test (gate 10.1a): a frozen
    // copy of the schema as it was BEFORE the Phase 10 EnvironmentUpdate
    // additions, kept under tests/fixtures/. Bindings from it prove the
    // pre-10.1 reader ignores environment messages/fields. The fixture
    // must never be edited to match the live schema — that would void
    // the test.
    let v1_schema = manifest_dir.join("tests/fixtures/cdda_ipc_v1.fbs");
    println!("cargo:rerun-if-changed={}", v1_schema.display());
    let v1_out = out_dir.join("v1");
    fs::create_dir_all(&v1_out).unwrap();
    run_flatc(&flatc, &v1_schema, &v1_out);
    fs::rename(
        v1_out.join("cdda_ipc_v1_generated.rs"),
        out_dir.join("cdda_ipc_v1_generated.rs"),
    )
    .expect("rename v1 generated file");
}
