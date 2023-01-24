use walkdir::WalkDir;

use crate::Error;
use std::path::{Path, PathBuf};

pub(crate) fn search_file(path: &Path, search_dirs: &[&Path]) -> Result<PathBuf, crate::Error> {
    if path.is_absolute() && path.exists() {
        return Ok(path.to_owned());
    }

    struct SearchResult {
        path: PathBuf,
        score: usize,
    }
    let mut results = Vec::new();

    for search_dir in search_dirs {
        let normalized_path = path
            .file_stem()
            .ok_or_else(|| Error::NoFileStem(path.to_owned()))?;
        let matching_extension = path.extension().map(|ext| ext.to_str()).flatten();
        let ancestors = path
            .ancestors()
            .skip(1)
            .map(Path::to_owned)
            .collect::<Vec<_>>();

        for entry in WalkDir::new(search_dir).follow_links(true) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => break,
            };

            let entry_extension = entry.path().extension().map(|e| e.to_str()).flatten();
            let mut score = 0;
            if entry_extension == matching_extension {
                score += 100;
            }

            let stem = entry.path().file_stem();
            if stem == Some(normalized_path) {
                score += 1000;
            } else if stem
                .map(|stem| stem.eq_ignore_ascii_case(normalized_path))
                .unwrap_or(false)
            {
                score += 750;
            } else {
                // absolutely not a match
                continue;
            }

            results.push(SearchResult {
                path: entry.path().to_path_buf(),
                score,
            });
        }
    }

    results
        .into_iter()
        .max_by_key(|r| r.score)
        .map(|r| r.path)
        .ok_or_else(|| Error::FileNotFound(path.to_string_lossy().into_owned()))
}
