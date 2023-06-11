use normpath::PathExt;
use std::path::Path;
use std::path::PathBuf;

use crate::doom_dir;
use crate::error::Error;

pub(crate) fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        doom_dir()?
            .join(path)
            .normalize()
            .map_err(Error::Io)?
            .into()
    };

    Ok(absolute_path)
}
