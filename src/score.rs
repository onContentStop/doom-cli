use itertools::Itertools;

use crate::error::Error;

use std::path::PathBuf;

pub(crate) fn score_entry(
    entry: &walkdir::DirEntry,
    base_name: &std::ffi::OsStr,
    extension: Option<&std::ffi::OsStr>,
    entry_extension: &str,
    ancestors: &[PathBuf],
) -> Result<usize, Error> {
    let mut score = 0;
    let stem = entry
        .path()
        .file_stem()
        .ok_or_else(|| Error::NoFileStem(entry.path().to_string_lossy().into_owned()))?;
    let stems_eq = stem
        .to_string_lossy()
        .eq_ignore_ascii_case(base_name.to_string_lossy().as_ref());
    let stems_case_eq = stem.to_string_lossy() == base_name.to_string_lossy();
    let extensions_match = extension
        .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case(entry_extension))
        .unwrap_or(true);
    let ancestors_eq = ancestors
        .iter()
        .zip(entry.path().ancestors().skip(1))
        .all_equal();
    if stems_eq {
        // doom2
        score += 2;
    }
    if stems_case_eq {
        // DOOM2
        score += 5;
    }
    if extensions_match {
        // Example.wad
        score += 1;
        if stems_eq {
            // doom2.wad
            score += 10;
        }
        if stems_case_eq {
            score += 5;
        }
    }
    if entry.path().is_dir() {
        // break ties with dirs and wads
        score /= 2;
    }
    if stems_eq && ancestors_eq {
        // iwad/doom2
        score += 20;
    }
    Ok(score)
}
