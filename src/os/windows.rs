//! Windows-specific implementations.

use crate::unit::{AddMode, Symlink};
use anyhow::{bail, Error};
use std::borrow::Cow;
use std::env::consts;
use std::path::{Path, PathBuf};

/// Convert into an executable path.
pub fn exe_path(mut path: PathBuf) -> PathBuf {
    if path.extension() == Some(consts::EXE_EXTENSION.as_ref()) {
        return path;
    }

    path.set_extension(consts::EXE_EXTENSION);
    path
}

/// Convert the given command into a path.
///
/// This adds the platform-specific extension for Windows.
pub fn command<'a>(base: &'a str) -> Cow<'a, Path> {
    Cow::from(exe_path(PathBuf::from(base)))
}
