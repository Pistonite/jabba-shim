//! Snapshot tests for the `jabba` shim `--help` output.
//!
//! Each `test_snapshot!` invocation runs the built `jabba` binary with the given args,
//! captures stdout, and compares it against a checked-in golden file under
//! `tests/help_snapshots/<name>.expected`. Run with `UPDATE_SNAPSHOTS=1` to (re)generate
//! the golden files.

use std::path::PathBuf;
use std::process::Command;

/// Define a snapshot test case. The name is an identifier used both as the `#[test]`
/// function name and as the snapshot file basename.
macro_rules! test_snapshot {
    ($name:ident, [$($arg:expr),* $(,)?]) => {
        #[test]
        fn $name() {
            run_snapshot(stringify!($name), &[$($arg),*]);
        }
    };
}

test_snapshot!(help, ["--help"]);
test_snapshot!(help_short, ["-h"]);
test_snapshot!(alias_help, ["alias", "--help"]);
test_snapshot!(current_help, ["current", "--help"]);
test_snapshot!(install_help, ["install", "--help"]);
test_snapshot!(link_help, ["link", "--help"]);
test_snapshot!(ls_help, ["ls", "--help"]);
test_snapshot!(ls_alias_help, ["ls-alias", "--help"]);
test_snapshot!(ls_remote_help, ["ls-remote", "--help"]);
test_snapshot!(unalias_help, ["unalias", "--help"]);
test_snapshot!(uninstall_help, ["uninstall", "--help"]);
test_snapshot!(unlink_help, ["unlink", "--help"]);
test_snapshot!(use_help, ["use", "--help"]);
test_snapshot!(which_help, ["which", "--help"]);

fn snapshot_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/help_snapshots"))
}

/// Normalize line endings and the trailing newline so cross-platform/editor differences
/// don't cause spurious mismatches.
fn normalize(s: &str) -> String {
    format!("{}\n", s.replace("\r\n", "\n").trim_end_matches('\n'))
}

fn run_snapshot(name: &str, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_jabba"))
        .args(args)
        .env("NO_COLOR", "1")
        .env("COLUMNS", "100")
        .output()
        .unwrap_or_else(|e| panic!("failed to run jabba binary: {e}"));

    let actual = normalize(&String::from_utf8_lossy(&output.stdout));

    let dir = snapshot_dir();
    std::fs::create_dir_all(&dir).expect("failed to create snapshot dir");
    let expected_path = dir.join(format!("{name}.expected"));
    let actual_path = dir.join(format!("{name}.actual"));

    // Always leave the observed output on disk for inspection.
    std::fs::write(&actual_path, &actual).expect("failed to write .actual snapshot");

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&expected_path, &actual).expect("failed to write .expected snapshot");
        return;
    }

    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
        panic!("missing snapshot {name}.expected; run with UPDATE_SNAPSHOTS=1 to create it")
    });
    let expected = normalize(&expected);

    if actual != expected {
        panic!(
            "snapshot `{name}` mismatch (run with UPDATE_SNAPSHOTS=1 to update)\n\
             --- expected ---\n{expected}\n--- actual ---\n{actual}"
        );
    }
}
