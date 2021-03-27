use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::util::absolute_path;
use crate::Error;

#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub(crate) enum DoomEngineKind {
    Vanilla,
    Boom,
    MBF,
    Eternity,
    ZDoom,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct DoomEngine {
    aliases: Vec<String>,
    pub binary: PathBuf,
    pub kind: DoomEngineKind,
    pub supports_widescreen_assets: bool,
    pub required_args: Vec<String>,
}

pub(crate) struct KnownEngines {
    keys: HashMap<String, usize>,
    engines: Vec<DoomEngine>,
}

impl KnownEngines {
    pub fn new(engines: Vec<DoomEngine>) -> Self {
        Self {
            keys: engines
                .iter()
                .enumerate()
                .flat_map(|(i, e)| e.aliases.iter().map(move |a| (a.clone(), i)))
                .collect(),
            engines,
        }
    }

    pub fn get(&self, alias: &str) -> Option<&DoomEngine> {
        let index = *self.keys.get(alias)?;
        Some(&self.engines[index])
    }
}

pub(crate) fn read_known_engines() -> Result<KnownEngines, Error> {
    let engines_json_path = crate::doom_dir().map(|d| d.join("engines.json"))?;
    println!("Searching for Doom engine definitions in {}", engines_json_path.to_string_lossy());
    if !engines_json_path.exists() {
        println!("Path not found, creating template. Please fill out this template.");
        let mut f = File::create(&engines_json_path)?;

        use std::io::Write;
        write!(
            f,
            "{}",
            r#"
[
    {
        "aliases": ["example", "ex"],
        "binary": "/dev/null",
        "kind": "Vanilla",
        "supports_widescreen_assets": false,
        "required_args": []
    }
]
        "#
            .trim()
        )?;
    }

    let engines: Vec<DoomEngine> = serde_json::from_reader(File::open(&engines_json_path)?)
        .map_err(|error| Error::BadJson {
            file: engines_json_path,
            error,
        })?;
    let engines: Vec<DoomEngine> = engines
        .into_iter()
        .map(|mut engine| {
            absolute_path(engine.binary.clone()).map(|binary| {
                engine.binary = binary;
                engine
            })
        })
        .collect::<Result<_, _>>()?;
    println!("Found engines:");
    engines.iter().for_each(|eng| println!("    {}", eng.aliases[0]));
    Ok(KnownEngines::new(engines))
}
