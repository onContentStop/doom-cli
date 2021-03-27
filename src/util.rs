use std::path::Path;
use std::path::PathBuf;
use path_clean::PathClean;

use crate::doom_dir;
use crate::Error;

pub(crate) fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        doom_dir()?.join(path).clean()
    };

    Ok(absolute_path)
}
