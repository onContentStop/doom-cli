use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use crate::doom_dir;
use crate::error::Error;
use crate::pwads::Pwads;
use crate::search::search_files;
use crate::FileType;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub(crate) struct Autoloads {
    pub(crate) universal: Vec<String>,
    pub(crate) sourceport: HashMap<String, Vec<String>>,
    pub(crate) iwad: HashMap<String, Vec<String>>,
}

pub(crate) fn autoload(
    pwads: &mut Pwads,
    engine: impl AsRef<Path>,
    iwad: &str,
) -> Result<(), Error> {
    let autoload_path = doom_dir()?.join("autoloads.ron");
    File::open(&autoload_path).or_else(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            write!(
                File::create(&autoload_path).map_err(|e| { Error::CreatingAutoloadsFile(e) })?,
                indoc! {r#"
                    Autoloads(
                        // Place here those PWADs you always want to load.
                        universal: [],
                        iwad: {{
                            // Place in here those PWADs that only load based on the IWAD.
                            "doom2.wad": ["foo.wad"],
                        }},
                        sourceport: {{
                            // Place in here those PWADs that only load based on the sourceport.
                            "example": ["bar.pk3"],
                        }},
                    )
                "#},
            )
            .map_err(Error::Io)?;
            File::open(autoload_path.as_path()).map_err(Error::OpeningFile)
        } else {
            Err(Error::Io(e))
        }
    })?;
    let autoloads: Autoloads = ron::from_str(
        String::from_utf8_lossy(
            std::fs::read(autoload_path.as_path())
                .map_err(Error::Io)?
                .as_slice(),
        )
        .as_ref(),
    )
    .map_err(|e| Error::BadRon {
        file: autoload_path.clone(),
        error: e,
    })?;

    let universal_pwads = search_files(&autoloads.universal, FileType::Pwad)?;
    pwads.add_wads(universal_pwads);

    autoloads
        .sourceport
        .get(
            engine
                .as_ref()
                .file_stem()
                .ok_or_else(|| Error::NoFileStem(engine.as_ref().to_string_lossy().to_string()))?
                .to_string_lossy()
                .as_ref(),
        )
        .map(|engine_specific_pwads| {
            pwads.add_wads(search_files(engine_specific_pwads, FileType::Pwad)?);
            Result::<(), Error>::Ok(())
        })
        .unwrap_or(Ok(()))?;
    if let Some(iwad_specific_pwads) = autoloads.iwad.get(iwad) {
        pwads.add_wads(search_files(iwad_specific_pwads, FileType::Pwad)?);
    }
    Ok(())
}
