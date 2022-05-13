use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Io(std::io::Error),
    #[error("No Doom directory exists or is configured")]
    NoDoomDir,
    #[error("No engines have been configured yet. Please add them to {0}.")]
    NoEngines(PathBuf),
    #[error(transparent)]
    Hjson(deser_hjson::Error),
    #[error("No file stem in '{0}'")]
    NoFileStem(PathBuf),
}
