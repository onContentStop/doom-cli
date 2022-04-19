use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Error;

mod alias_map;

use alias_map::AliasMap;

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub(crate) enum EngineKind {
    Vanilla,
    Boom,
    Mbf,
    Eternity,
    ZDoom,
}

#[derive(Deserialize)]
struct EnginesFile {
    engines: HashMap<String, RawEngine>,
}

#[derive(Deserialize)]
struct RawEngine {
    aliases: Vec<String>,
    path: PathBuf,
    kind: EngineKind,
    supports_widescreen_assets: Option<bool>,
    required_args: Option<Vec<String>>,
}

#[derive(Clone)]
pub(crate) struct Engine {
    pub(crate) path: PathBuf,
    pub(crate) kind: EngineKind,
    pub(crate) supports_widescreen_assets: bool,
    pub(crate) required_args: Vec<String>,
}

pub(crate) struct Engines {
    first: Option<String>,
    data: AliasMap<String, Engine>,
}

impl Engines {
    pub(crate) fn read_from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(Error::Io)?;
        let raw: EnginesFile = deser_hjson::from_str(&contents).map_err(Error::Hjson)?;
        let mut engines = Engines {
            first: None,
            data: AliasMap::new(),
        };
        for (k, v) in raw.engines.into_iter() {
            if engines.first == None {
                engines.first = Some(k.clone());
            }
            engines.data.insert(
                k.clone(),
                Engine {
                    path: v.path,
                    kind: v.kind,
                    supports_widescreen_assets: v.supports_widescreen_assets.unwrap_or(false),
                    required_args: v.required_args.unwrap_or(Vec::new()),
                },
            );
            for alias in v.aliases {
                engines.data.alias(&k, alias).unwrap();
            }
        }

        Ok(engines)
    }

    pub(crate) fn first(&self) -> Option<&Engine> {
        self.first.as_ref().and_then(|f| self.data.get(f))
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Engine> {
        self.data.get(name)
    }
}

pub(crate) fn create_template(engines_file_path: impl AsRef<Path>) -> Result<(), Error> {
    std::fs::write(
        engines_file_path,
        r#"
{
  engines: {
    // example: {
    //   aliases: ["ex"],
    //   path: "/dev/zero",
    //   kind: Mbf,
    // }
  }
}
"#
        .trim(),
    )
    .map_err(Error::Io)
}
