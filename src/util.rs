use normpath::PathExt;
use std::path::Path;
use std::path::PathBuf;

use crate::doom_dir;
use crate::error::Error;

#[cfg(windows)]
fn fix_slashes(path: impl AsRef<Path>) -> PathBuf {
    use std::{ffi::OsString, os::windows::prelude::OsStrExt, os::windows::prelude::OsStringExt};

    let converted = path
        .as_ref()
        .as_os_str()
        .encode_wide()
        .into_iter()
        .map(|b| {
            const FORWARD_SLASH: u16 = b'/' as u16;
            match b {
                FORWARD_SLASH => b'\\' as u16,
                b => b,
            }
        })
        .collect::<Vec<_>>();
    OsString::from_wide(&converted).into()
}

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

    if cfg!(windows) {
        Ok(fix_slashes(absolute_path))
    } else {
        Ok(absolute_path)
    }
}
