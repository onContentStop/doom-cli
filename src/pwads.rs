use std::path::Path;
use std::path::PathBuf;

pub(crate) struct Pwads {
    wads: Vec<PathBuf>,
    dehs: Vec<PathBuf>,
}

impl Pwads {
    pub(crate) fn new() -> Self {
        Self {
            wads: vec![],
            dehs: vec![],
        }
    }

    pub(crate) fn add_wads(&mut self, mut wads: Vec<PathBuf>) {
        self.wads.append(&mut wads);
    }

    pub(crate) fn add_wad(&mut self, wad: impl AsRef<Path>) {
        self.wads.push(wad.as_ref().to_owned());
    }

    pub(crate) fn add_dehs(&mut self, mut dehs: Vec<PathBuf>) {
        self.dehs.append(&mut dehs);
    }

    pub(crate) fn add_deh(&mut self, deh: PathBuf) {
        self.dehs.push(deh);
    }

    pub(crate) fn wads(&self) -> &[PathBuf] {
        &self.wads
    }

    pub(crate) fn dehs(&self) -> &[PathBuf] {
        &self.dehs
    }
}
