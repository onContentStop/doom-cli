use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use indoc::indoc;
use log::info;
use log::trace;
use log::warn;
use serde::Deserialize;
use serde::Serialize;

const EXAMPLE_ENGINES_FILE: &str = indoc!(
    r#"
    // Replace 'example' with the name of your sourceport.
    example {
        // Put here any aliases you want to use with the -e option.
        aliases example ex
        // Path to the binary
        binary /dev/zero
        // What compatibility levels does this engine support?
        // Valid values: {Vanilla, Boom, MBF, Eternity, ZDoom}
        kind Vanilla
        // Does this engine support the official Doom widescreen assets?
        // Most engines don't, so if you don't know then put false here.
        supports_widescreen_assets false
        // Are there any extra arguments that should always be passed to the engine?
        required_args
    }
    "#
);

use crate::util::absolute_path;
use crate::Error;

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum DoomEngineKind {
    Vanilla,
    Boom,
    MBF,
    Eternity,
    ZDoom,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct DoomEngine {
    aliases: Vec<String>,
    pub binary: PathBuf,
    pub kind: DoomEngineKind,
    pub supports_widescreen_assets: bool,
    pub required_args: Vec<String>,
}

impl Default for DoomEngine {
    fn default() -> Self {
        Self {
            aliases: Vec::new(),
            binary: PathBuf::from("/bin/true"),
            kind: DoomEngineKind::Vanilla,
            supports_widescreen_assets: false,
            required_args: Vec::new(),
        }
    }
}

pub(crate) struct KnownEngines {
    alias_map: HashMap<String, usize>,
    engines: Vec<DoomEngine>,
}

pub(crate) struct KnownEnginesIterator {
    iter: Box<dyn Iterator<Item = String>>,
}

impl Iterator for KnownEnginesIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl KnownEngines {
    pub fn new(engine_map: HashMap<String, DoomEngine>) -> Self {
        let mut alias_map = HashMap::new();
        let mut engines = Vec::new();
        for (name, eng) in engine_map {
            let i = engines.len();
            alias_map.insert(name, i);
            for alias in eng.aliases.iter() {
                alias_map.insert(alias.clone(), i);
            }
            engines.push(eng);
        }
        Self { alias_map, engines }
    }

    pub fn get(&self, name: &str) -> Option<&DoomEngine> {
        let index = *self.alias_map.get(name)?;
        Some(&self.engines[index])
    }

    pub fn iter(&'_ self) -> KnownEnginesIterator {
        let engines = self.engines.clone();
        KnownEnginesIterator {
            iter: Box::new(
                engines
                    .into_iter()
                    .map(|e| e.aliases)
                    .flat_map(|ss| ss.into_iter()),
            ),
        }
    }
}

pub(crate) fn read_known_engines() -> Result<KnownEngines, Error> {
    let engines_json_path = crate::doom_dir()?.join("engines.kdl");
    trace!(
        "Searching for Doom engine definitions in {}",
        engines_json_path.to_string_lossy()
    );
    if !engines_json_path.exists() {
        warn!("Path not found, creating template. Please fill out this template.");
        let mut f = File::create(&engines_json_path).map_err(Error::Io)?;

        use std::io::Write;
        write!(f, "{}", EXAMPLE_ENGINES_FILE).map_err(Error::Io)?;
    }

    let engines_raw = kdl::parse_document(String::from_utf8_lossy(
        &std::fs::read(engines_json_path.as_path()).map_err(Error::Io)?,
    ))
    .map_err(|error| Error::BadKdl {
        file: engines_json_path,
        error,
    })?;

    let engines = {
        let mut engines = HashMap::<String, DoomEngine>::new();
        for engine_raw in engines_raw {
            let mut engine = DoomEngine::default();
            for node in engine_raw.children {
                match node.name.as_str() {
                    "aliases" => engine
                        .aliases
                        .append(&mut node.values.into_iter().map(|v| v.to_string()).collect()),
                    "binary" => {
                        engine.binary = PathBuf::from(
                            node.values
                                .into_iter()
                                .map(|v| v.to_string())
                                .next()
                                .unwrap(),
                        )
                    }
                    "kind" => {
                        engine.kind = match node
                            .values
                            .into_iter()
                            .map(|v| v.to_string())
                            .next()
                            .unwrap()
                            .as_ref()
                        {
                            "Vanilla" => DoomEngineKind::Vanilla,
                            "MBF" => DoomEngineKind::MBF,
                            "Boom" => DoomEngineKind::Boom,
                            "ZDoom" => DoomEngineKind::ZDoom,
                            "Eternity" => DoomEngineKind::Eternity,
                            s => panic!("bad engine kind: {}", s),
                        }
                    }
                    "supports_widescreen_assets" => {
                        engine.supports_widescreen_assets = node
                            .values
                            .into_iter()
                            .map(|v| v.to_string())
                            .next()
                            .unwrap()
                            == "true";
                    }
                    "required_args" => {
                        engine.required_args =
                            node.values.into_iter().map(|v| v.to_string()).collect();
                    }
                    _ => {}
                }
            }
            engines.insert(engine_raw.name, engine);
        }
        engines
    };

    let engines: HashMap<String, DoomEngine> = engines
        .into_iter()
        .map(|(name, mut engine)| {
            absolute_path(engine.binary.clone()).map(|binary| {
                engine.binary = binary;
                (name, engine)
            })
        })
        .collect::<Result<_, _>>()?;
    info!("Found engines:");
    engines.keys().for_each(|eng| info!("    {}", eng));
    Ok(KnownEngines::new(engines))
}
