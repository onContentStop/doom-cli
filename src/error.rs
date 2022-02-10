#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Io(std::io::Error),
    #[error("No Doom directory exists or is configured")]
    NoDoomDir,
    #[error("No engines have been configured yet")]
    NoEngines,
    #[error(transparent)]
    Hjson(deser_hjson::Error),
}
