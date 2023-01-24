use std::path::Path;
use std::path::PathBuf;

use crate::error::Error;
use crate::search::search_file;
use crate::search::search_file_by;
use crate::FileType;
use crate::ARG_SEPARATOR;

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

pub(crate) fn parse_arg_pwads(
    arg_pwads_raw: &str,
    viddump_folder_name: &mut Vec<String>,
    pwads: &mut Pwads,
) -> Result<(), Error> {
    let mut arg_pwads = vec![];
    for pwad in arg_pwads_raw.split(ARG_SEPARATOR) {
        let mut pwad_files = search_file_by(pwad, FileType::Pwad, |f| {
            f.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    ["wad", "pk3", "pk7", "pke", "zip", "deh", "bex"]
                        .contains(&ext.to_lowercase().as_str())
                })
                .unwrap_or(true)
        })?;
        viddump_folder_name.extend(
            search_file(pwad, FileType::Pwad)?
                .iter()
                .map(|p| {
                    p.file_stem()
                        .ok_or_else(|| Error::NoFileStem(p.to_string_lossy().into_owned()))
                        .and_then(|p| {
                            p.to_str()
                                .ok_or_else(|| Error::NonUtf8Path(p.to_string_lossy().into_owned()))
                        })
                        .map(|p| p.to_owned())
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        let i = if pwad_files.len() > 1 {
            dialoguer::Select::new()
                .items(
                    pwad_files
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                        .as_ref(),
                )
                .with_prompt(
                    format!("Multiple results were found for {}. Select one.", pwad).as_str(),
                )
                .interact()
                .map_err(Error::Io)?
        } else {
            0
        };
        arg_pwads.push(pwad_files.remove(i));
    }
    for pwad in arg_pwads {
        match pwad
            .extension()
            .map(|ext| {
                ext.to_str()
                    .ok_or_else(|| Error::NonUtf8Path(ext.to_string_lossy().into_owned()))
            })
            .transpose()?
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "wad" | "pk3" | "zip" | "pk7" | "pke" | "" => pwads.add_wad(pwad),
            "deh" | "bex" => pwads.add_deh(pwad),
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub(crate) fn parse_extra_pwads(extra_pwads_raw: &str, pwads: &mut Pwads) -> Result<(), Error> {
    for pwad in extra_pwads_raw.split(ARG_SEPARATOR) {
        let mut found = search_file(pwad, FileType::Pwad)?;
        let i = if found.len() > 1 {
            dialoguer::Select::new()
                .items(
                    &found
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>(),
                )
                .with_prompt("Multiple candidates were found. Select one.")
                .interact()
                .map_err(Error::Io)?
        } else {
            0
        };
        pwads.add_wad(found.remove(i));
    }
    Ok(())
}
