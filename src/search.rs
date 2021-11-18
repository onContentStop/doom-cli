use crate::error::Error;
use crate::score::score_entry;
use crate::util::absolute_path;
use crate::FileType;
use itertools::Itertools;
use log::info;
use log::trace;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub(crate) fn search_files(list: &[String], ty: FileType) -> Result<Vec<PathBuf>, Error> {
    list.iter()
        .map(move |i| {
            search_file_in_dirs_by(PathBuf::from(i), ty.get_search_dirs()?, |p| {
                ["wad", "deh", "bex", "pk3", "pk7", "pke", "zip"].contains(
                    &p.extension()
                        .map(|ext| ext.to_string_lossy().to_string())
                        .unwrap_or_default()
                        .as_str(),
                )
            })
        })
        .map(|rr| rr.map(|r| r.into_iter().next().unwrap()))
        .collect()
}

pub(crate) fn search_file(name: impl AsRef<str>, ty: FileType) -> Result<Vec<PathBuf>, Error> {
    search_file_in_dirs_by(name.as_ref().into(), ty.get_search_dirs()?, |_| true)
}

pub(crate) fn search_file_by(
    name: impl AsRef<str>,
    ty: FileType,
    predicate: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>, Error> {
    search_file_in_dirs_by(name.as_ref().into(), ty.get_search_dirs()?, predicate)
}

pub(crate) fn search_file_in_dirs_by(
    name: PathBuf,
    search_dirs: Vec<PathBuf>,
    predicate: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>, Error> {
    if name.is_absolute() {
        let mut parent = name.clone();
        parent.pop();
        search_file_in_dirs_by(
            PathBuf::from(
                name.file_stem()
                    .ok_or_else(|| Error::NoFileStem(name.to_string_lossy().into_owned()))?,
            ),
            vec![parent],
            predicate,
        )
    } else {
        for search_dir in search_dirs {
            info!(
                "Searching for '{}' in '{}'",
                name.to_string_lossy(),
                search_dir.to_string_lossy()
            );

            let base_name = name
                .file_stem()
                .ok_or_else(|| Error::NoFileStem(name.to_string_lossy().into_owned()))?;
            let extension = name.extension();
            let ancestors = name
                .ancestors()
                .skip(1)
                .map(|p| p.to_path_buf())
                .collect::<Vec<_>>();

            let search_dir = absolute_path(PathBuf::from(&search_dir))?;

            struct SearchResult {
                path: PathBuf,
                score: usize,
            }
            let mut results = vec![];

            for entry in WalkDir::new(search_dir).contents_first(true) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        info!("Stopping search due to an error: {}", e);
                        break;
                    }
                };

                if entry.path().is_dir() {
                    continue;
                }

                if !predicate(entry.path()) {
                    continue;
                }

                let entry_extension = entry
                    .path()
                    .extension()
                    .map(|e| {
                        e.to_str().ok_or_else(|| {
                            Error::NonUtf8Path(entry.path().to_string_lossy().into_owned())
                        })
                    })
                    .transpose()?
                    .unwrap_or("");

                let entry_score =
                    score_entry(&entry, base_name, extension, entry_extension, &ancestors)?;
                if entry_score > 1 {
                    results.push(SearchResult {
                        path: entry.path().into(),
                        score: entry_score,
                    });
                }
            }

            if !results.is_empty() {
                let results = results
                    .into_iter()
                    .sorted_by_key(|r| r.score)
                    .map(|r| r.path)
                    .rev()
                    .collect::<Vec<_>>();
                trace!(
                    "Results: [{}]",
                    results
                        .iter()
                        .map(|r| r.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return Ok(results);
            }
        }
        Err(Error::FileNotFound(name.to_string_lossy().into_owned()))
    }
}
