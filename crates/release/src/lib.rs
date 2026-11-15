//! Phase 8 release machinery: the parts of the release gate that
//! are pure logic over the protocol — crash reporting (8.13),
//! install/upgrade stamps (8.11/8.12), and the scripted soak
//! driver (8.8/8.9) that plays dummy sessions through the Phase 8
//! stores.
//!
//! Everything here is engine-independent on purpose: the release
//! checks must be able to run headless, on a fresh install, and in
//! CI, none of which have a GPU or a window. The live-loop halves
//! (drawing the overmap over the 3D view, soak frame times) ride
//! the Phase 2h frame surface; the data halves are gated here.

pub mod package;
pub mod report;
pub mod soak;

pub use package::{check_asset_cache, write_stamp, CacheVerdict, PackageStamp};
pub use report::{format_report, install_panic_hook, write_report, CrashReport, FORK_COMMIT};
pub use soak::{run_soak, SoakConfig, SoakStats};
