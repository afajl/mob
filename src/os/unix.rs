//! Unix-specific implementations.

use std::borrow::Cow;
use std::path::Path;

/// Convert the given command into a path.
///
/// This adds the platform-specific extension for Windows.
pub fn command(base: &str) -> Cow<'_, Path> {
    Cow::from(Path::new(base))
}
