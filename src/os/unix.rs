//! Unix-specific implementations.

use anyhow::Error;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Convert into an executable path.
pub fn exe_path(path: PathBuf) -> PathBuf {
    path
}

/// Convert the given command into a path.
///
/// This adds the platform-specific extension for Windows.
pub fn command(base: &str) -> Cow<'_, Path> {
    Cow::from(Path::new(base))
}

/// Detect git command.
#[allow(unused)]
pub fn detect_git() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("git"))
}
