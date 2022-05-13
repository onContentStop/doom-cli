use walkdir::WalkDir;

use crate::Error;
use std::path::{Path, PathBuf};

pub(crate) fn search_file(path: &Path, search_dirs: &[&Path]) -> Result<PathBuf, crate::Error> {
    if path.is_absolute() && path.exists() {
        return Ok(path.to_owned());
    }

    for search_dir in search_dirs {
        let normalized_path = path
            .file_stem()
            .ok_or_else(|| Error::NoFileStem(path.to_owned()))?;
        let matching_extension = path.extension();
        let ancestors = path
            .ancestors()
            .skip(1)
            .map(Path::to_owned)
            .collect::<Vec<_>>();

        struct SearchResult {
            path: PathBuf,
            score: usize,
        }
        let mut results = Vec::new();

        for entry in WalkDir::new(search_dir).contents_first(true) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => break,
            };

            if entry.path().is_dir() {
                continue;
            }

            let entry_extension = match entry.path().extension().map(|e| e.to_str()) {
                Some(ext) => ext,
                // no extension? skip it
                None => continue,
            };
        }
    }

    Ok(())
}
