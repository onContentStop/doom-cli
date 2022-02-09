#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Io(std::io::Error),
    #[error("No Doom directory exists or is configured")]
    NoDoomDir,
}
