//! Test helpers shared by the crate's unit tests.

use std::path::PathBuf;

use tempfile::TempDir;

/// A temporary directory inside the build tree.
///
/// `tempfile::tempdir()` writes to the machine's temp directory, which is a
/// shared, permission-managed location: on a locked-down machine, or inside a
/// sandbox that grants temp access per working directory, it fails for reasons
/// that have nothing to do with the code under test. Putting test scratch space
/// next to the build output keeps tests hermetic and always writable.
pub fn temp_dir() -> TempDir {
    let root = workspace_root().join("target").join("test-tmp");
    std::fs::create_dir_all(&root).expect("could not create the test temp root");
    TempDir::new_in(&root).expect("could not create a test temp directory")
}

fn workspace_root() -> PathBuf {
    // `CARGO_MANIFEST_DIR` points at `crates/refiny-core`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("the crate lives inside the workspace")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dirs_are_created_inside_the_build_tree() {
        let dir = temp_dir();
        assert!(dir.path().starts_with(workspace_root().join("target")));
        assert!(dir.path().is_dir());
    }

    #[test]
    fn each_call_gets_its_own_directory() {
        let first = temp_dir();
        let second = temp_dir();
        assert_ne!(first.path(), second.path());
    }
}
