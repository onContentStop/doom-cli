use std::path::{Path, PathBuf};

use crate::error::Error;

pub(crate) fn absolute_path(path: &Path, fallback_parent: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        fallback_parent.join(path)
    }
}
