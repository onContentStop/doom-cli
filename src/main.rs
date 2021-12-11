use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use std::str::FromStr;
use std::sync::Mutex;

use clap::App;
use clap::AppSettings;
use clap::Arg;
use dialoguer::console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use dialoguer::Input;
use dialoguer::MultiSelect;
use itertools::Itertools;
use log::error;
use log::info;
use log::warn;
use once_cell::sync::Lazy;

use crate::cmd::CommandLine;
use crate::cmd::Line;
use crate::engine::read_known_engines;
use crate::engine::DoomEngineKind;
use crate::error::Error;
use crate::pwads::parse_arg_pwads;
use crate::pwads::parse_extra_pwads;
use crate::pwads::Pwads;
use crate::render::batch_render;
use crate::util::absolute_path;

mod autoload;
mod cmd;
mod engine;
mod error;
mod job;
mod pwads;
mod render;
mod score;
mod search;
mod util;

static CUSTOM_DOOM_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

enum FileType {
    Iwad,
    Pwad,
    Demo,
}

impl FileType {
    fn get_search_dirs(&self) -> Result<Vec<PathBuf>, Error> {
        vec![doom_dir(), Ok(public_doom_dir())]
            .into_iter()
            .collect()
    }
}

const ARG_SEPARATOR: char = ',';

fn home_dir() -> Result<PathBuf, Error> {
    dirs::home_dir().ok_or(Error::Homeless)
}

fn doom_dir() -> Result<PathBuf, Error> {
    if let Some(dir) = CUSTOM_DOOM_DIR.lock().unwrap().as_ref() {
        Ok(dir.clone())
    } else {
        home_dir().map(|h| h.join("doom"))
    }
}

fn public_doom_dir() -> PathBuf {
    PathBuf::from("/public/doom")
}

fn demo_dir() -> Result<PathBuf, Error> {
    doom_dir().map(|d| d.join("demo"))
}

#[cfg(unix)]
fn dump_dir() -> PathBuf {
    let raw_output = String::from_utf8(
        Command::new("findmnt")
            .arg("/dev/sdd1")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let second_line = raw_output.lines().nth(1).unwrap_or_else(|| {
        error!("Please mount /dev/sdd1. I beg you.");
        exit(-1);
    });
    second_line.split_whitespace().next().unwrap().into()
}

#[cfg(windows)]
fn dump_dir() -> PathBuf {
    PathBuf::from("E:").join("Videos")
}

fn select_between<P: AsRef<Path>>(
    search: impl AsRef<str>,
    options: impl AsRef<[P]>,
) -> Result<Vec<PathBuf>, Error> {
    MultiSelect::new()
        .with_prompt(format!("Multiple files were found for the search term {}. Please select one or more of the following:", search.as_ref()))
        .items(
            &options
                .as_ref()
                .iter()
                .map(|opt| opt.as_ref().to_string_lossy())
                .collect::<Vec<_>>(),
        )
        .interact()
        .map(|indices| indices.iter().map(|i| options.as_ref()[*i].as_ref().to_owned()).collect())
        .map_err(Error::Io)
}

fn run_doom<'l>(mut cmdline: impl Iterator<Item = &'l str>) -> Result<(), Error> {
    let binary = PathBuf::from(cmdline.next().unwrap());
    if !binary.exists() {
        return Err(Error::FileNotFound(binary.to_string_lossy().into_owned()));
    }
    let binary_dir = dirname(&binary);
    let args = cmdline
        .filter_map(|arg| {
            let trimmed = arg.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect::<Vec<_>>();
    Command::new(binary)
        .args(args)
        .current_dir(binary_dir)
        .status()
        .map(|_| ())
        .map_err(Error::RunningDoom)
}

fn dirname(binary: &Path) -> PathBuf {
    let mut d = binary.to_owned();
    d.pop();
    d
}

fn run() -> Result<(), Error> {
    let app = App::new("Command-line Doom launcher")
            .version("0.1.0")
            .before_help("This Doom launcher allows shortcuts to the many long-winded options that Doom engines accept.")
            .setting(AppSettings::TrailingVarArg)
            .setting(AppSettings::ColorAuto)
            .arg(Arg::with_name("3p").long("3p").help("Add the 3P Sound Pack"))
            .arg(Arg::with_name("compatibility-level").short("c").long("compatibility-level").help("Set the compatibility level to LEVEL").value_name("LEVEL"))
            .arg(Arg::with_name("debug").short("G").long("debug").help("Run Doom under a debugger"))
            .arg(Arg::with_name("doom-dir").long("doom-dir").help("Set a custom Doom configuration directory"))
            .arg(Arg::with_name("engine").short("e").long("engine").help("Play the game with ENGINE instead of DSDA Doom").value_name("ENGINE"))
            .arg(Arg::with_name("extra-pwads").short("x").long("extra-pwads").help("Add PWADS to the game, silently").long_help("Silently means that when rendering a demo (with --render), the program will not add these PWADs to the folder name.").value_name("WAD").multiple(true))
            .arg(Arg::with_name("fast").short("f").long("fast").help("Enable fast monsters"))
            .arg(Arg::with_name("geometry").short("g").long("geometry").help("Set the screen resolution to WxH").long_help("Set the screen resolution to WxH; only supported on Boom-derived sourceports.").value_name("GEOM"))
            .arg(Arg::with_name("iwad").short("i").long("iwad").help("Set the game's IWAD").value_name("WAD"))
            .arg(Arg::with_name("no-confirm").long("no-confirm").short("n").help("Don't ask for confirmation before running Doom"))
            .arg(Arg::with_name("no-monsters").long("no-monsters").help("Play the game with no monsters"))
            .arg(Arg::with_name("pistol-start").long("pistol-start").help("Play each level from a pistol start").long_help("Play each level from a pistol start. Currently only works with Crispy Doom and PrBoom+."))
            .arg(Arg::with_name("play-demo").short("d").long("play-demo").help("Play back DEMO").value_name("DEMO"))
            .arg(Arg::with_name("pwads").short("p").long("pwads").help("Add PWADS to the game").multiple(true).value_name("WAD"))
            .arg(Arg::with_name("record").short("r").long("record").help("Record a demo to DEMO").value_name("DEMO").long_help("Record a demo to DEMO, relative to ~/doom/demo."))
            .arg(Arg::with_name("record-from-to").long("record-from-to").number_of_values(2).help("Play back FROM, allowing you to rewrite its ending to TO").long_help("Play FROM. You are allowed to press the join key at any time to begin recording your inputs from the current frame. Whenever you quit the game, the final result will be written to TO.").value_names(&["FROM", "TO"]))
            .arg(Arg::with_name("render").short("R").long("render").help("Render a demo as a video").long_help("The video will be placed in /extra/Videos/{iwad}/{pwads}/{demoname}.").value_name("DEMO"))
            .arg(Arg::with_name("respawn").long("respawn").help("Enable respawning monsters"))
            .arg(Arg::with_name("short-tics").long("short-tics").help("Play the game with short tics instead of long tics"))
            .arg(Arg::with_name("skill").short("s").long("skill").help("Set the game's skill level by a number").value_name("SKILL"))
            .arg(Arg::with_name("vanilla-weapons").long("vanilla-weapons").help("Load the game with smooth weapon animations"))
            .arg(Arg::with_name("video-mode").short("v").long("video-mode").help("Set the video mode of the game (software, hardware)").long_help("Only supported on Boom-derived sourceports.").value_name("MODE"))
            .arg(Arg::with_name("warp").short("w").long("warp").help("Start the game at a specific level number").value_name("LEVEL"))
            .arg(Arg::with_name("passthrough").multiple(true))
            ;

    let matches = app.get_matches();

    if let Some(doom_dir) = matches.value_of("doom-dir") {
        *CUSTOM_DOOM_DIR.lock().unwrap() = Some(PathBuf::from_str(doom_dir).unwrap());
    }

    if !doom_dir()?.exists() {
        let answer = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "You don't have a dedicated Doom directory at {}. Create it?",
                doom_dir()?.to_string_lossy()
            ))
            .interact()
            .map_err(Error::Io)?;
        if answer {
            create_dir_all(doom_dir()?).map_err(Error::Io)?;
            info!("Success.");
        } else {
            warn!("Cannot continue. You can set the dedicated Doom directory by passing the flag --doom-dir. You only have to pass the flag once, as it will be remembered.");
            return Ok(());
        }
    }

    let known_engines = read_known_engines()?;
    let engine_name = matches
        .value_of("engine")
        .map(|s| s.to_owned())
        .or_else(|| known_engines.iter().next())
        .ok_or(Error::NoEngines)?;
    let engine = &known_engines.get(&engine_name).unwrap_or_else(|| {
        error!("ERROR: Unknown sourceport '{}'", engine_name);
        exit(-1);
    });

    let mut search_iwads: Box<dyn Iterator<Item = String>> = matches
        .value_of("iwad")
        .map::<Box<dyn Iterator<Item = String>>, _>(|i| Box::new(std::iter::once(i.to_string())))
        .unwrap_or_else(|| {
            Box::new(
                ["DOOM2.WAD", "DOOM.WAD", "DOOMU.WAD", "DOOM1.WAD"]
                    .iter()
                    .map(|i: &&str| i.to_string()),
            )
        });
    let iwad_path = loop {
        let iwad = match search_iwads.next() {
            Some(i) => i,
            None => break None,
        };
        let iwad_path = search::search_file(&iwad, FileType::Iwad).or_else(|e| {
            if let Error::FileNotFound(_) = e {
                Ok(vec![])
            } else {
                Err(e)
            }
        })?;
        if iwad_path.is_empty() {
            warn!("IWAD not found: '{}'", iwad);
        } else {
            break Some(iwad_path);
        }
    };
    if iwad_path.is_none() {
        error!("No IWADs could be found.");
        exit(-1);
    }
    let iwad_path = iwad_path.unwrap();
    let iwad_path = absolute_path(&iwad_path[0])?;
    let iwad = iwad_path.to_string_lossy().to_string();

    let iwad_base = iwad_path
        .file_name()
        .ok_or_else(|| Error::NoFileStem(iwad_path.to_string_lossy().into_owned()))
        .and_then(|f| {
            f.to_str()
                .ok_or_else(|| Error::NonUtf8Path(f.to_string_lossy().into_owned()))
        })?;
    let iwad_noext = iwad_path
        .file_stem()
        .ok_or_else(|| Error::NoFileStem(iwad_path.to_string_lossy().into_owned()))
        .and_then(|i| {
            i.to_str()
                .ok_or_else(|| Error::NonUtf8Path(i.to_string_lossy().into_owned()))
        })?
        .to_lowercase();

    let mut cmdline = CommandLine::new();
    if matches.is_present("debug") {
        cmdline.push_line(Line::from_word("/usr/bin/lldb", 0));
    }
    cmdline.push_line(Line::from_word(
        engine
            .binary
            .to_str()
            .ok_or_else(|| Error::NonUtf8Path(engine.binary.to_string_lossy().into_owned()))?,
        0,
    ));
    if matches.is_present("debug") {
        cmdline.push_line(Line::from_word("--", 0));
    }
    if !engine.required_args.is_empty() {
        cmdline.push_line(Line::from_words(&engine.required_args, 1));
    }
    cmdline.push_line(Line::from_words(&["-iwad", &iwad], 1));

    let mut pwads = Pwads::new();

    if engine.supports_widescreen_assets {
        if let Ok(assets) = search::search_file(
            format!("{}_widescreen_assets.wad", iwad_noext),
            FileType::Pwad,
        ) {
            pwads.add_wads(assets);
        } else {
            warn!(
                "Couldn't find widescreen assets for {}.",
                match iwad_noext.as_str() {
                    "doom" => "Doom",
                    "doom2" => "Doom 2",
                    "tnt" => "TNT: Evilution",
                    "plutonia" => "The Plutonia Experiment",
                    _ => "<unknown IWAD>",
                }
            );
        }
    }

    autoload::autoload(&mut pwads, &engine.binary, &iwad_noext)?;

    let mut viddump_folder_name = vec![];

    if let Some(arg_pwads_raw) = matches.value_of("pwads") {
        parse_arg_pwads(arg_pwads_raw, &mut viddump_folder_name, &mut pwads)?;
    }

    if let Some(extra_pwads_raw) = matches.value_of("extra-pwads") {
        parse_extra_pwads(extra_pwads_raw, &mut pwads)?;
    }

    if matches.is_present("vanilla-weapons") {
        pwads.add_wads(search::search_file("vsmooth.wad", FileType::Pwad)?);
        pwads.add_dehs(search::search_file("vsmooth.deh", FileType::Pwad)?);
    }

    if matches.is_present("3p") {
        let sound_pack = search::search_file("3P Sound Pack.wad", FileType::Pwad)?;
        pwads.add_wad(&sound_pack[0]);
    }

    if !pwads.wads().is_empty() {
        cmdline.push_line(Line::from_word("-file", 1));
        pwads.wads().iter().try_for_each(|pwad| {
            pwad.to_str()
                .ok_or_else(|| Error::NonUtf8Path(pwad.to_string_lossy().into_owned()))
                .map(|pwad| cmdline.push_line(Line::from_word(pwad, 2)))
        })?;
    }

    if !pwads.dehs().is_empty() {
        cmdline.push_line(Line::from_word("-deh", 1));
        pwads.dehs().iter().try_for_each(|deh| {
            deh.to_str()
                .ok_or_else(|| Error::NonUtf8Path(deh.to_string_lossy().into_owned()))
                .map(|deh| cmdline.push_line(Line::from_word(deh, 2)))
        })?;
    }

    let complevel = matches.value_of("compatibility-level").unwrap_or("9");
    cmdline.push_line(Line::from_words(
        &[String::from("-complevel"), complevel.to_string()],
        1,
    ));

    if matches.is_present("pistol-start") {
        cmdline.push_line(Line::from_word("-pistolstart", 1));
    }

    let vidmode = matches.value_of("video-mode").unwrap_or("GL");
    cmdline.push_line(Line::from_words(&["-vidmode", vidmode], 1));

    let geom = matches.value_of("geometry").unwrap_or("2560x1440F");
    cmdline.push_line(Line::from_words(&["-geom", geom], 1));

    let skill_param = if engine.kind == DoomEngineKind::ZDoom {
        &["+skill", "3"]
    } else {
        &["-skill", "4"]
    };

    if let Some(recording_demo) = matches.value_of("record") {
        let demo_path = PathBuf::from(recording_demo);
        let demo_path = if demo_path.is_absolute() {
            demo_path
        } else {
            demo_dir()?.join(demo_path)
        };
        cmdline.push_line(Line::from_word("-record", 1));
        cmdline.push_line(Line::from_word(demo_path.to_string_lossy(), 2));
        if !matches.is_present("short-tics") {
            cmdline.push_line(Line::from_word("-longtics", 1));
        }
    } else if matches.is_present("short-tics") {
        cmdline.push_line(Line::from_word("-shorttics", 1));
    }

    if let Some(from_to) = matches.values_of("record-from-to") {
        let from_to = from_to.collect::<Vec<_>>();
        cmdline.push_line(Line::from_word("-recordfromto", 1));
        cmdline.push_line(Line::from_words(&from_to[0..2], 2));
    }

    if let Some(playing_demo) = matches.value_of("play-demo") {
        let demo = select_between(
            playing_demo,
            search::search_file(playing_demo, FileType::Demo)?,
        )?;
        if demo.is_empty() {
            error!("No such demo: {}", playing_demo);
            exit(-1);
        }
        cmdline.push_line(Line::from_word("-playdemo", 1));
        cmdline.push_line(Line::from_word(
            demo[0]
                .to_str()
                .ok_or_else(|| Error::NonUtf8Path(demo[0].to_string_lossy().into_owned()))?,
            2,
        ));
    }

    if let Some(warp) = matches.value_of("warp") {
        cmdline.push_line(Line::from_words(
            &{
                let mut words = vec!["-warp"];
                words.extend(warp.split(ARG_SEPARATOR));
                words
            },
            1,
        ));
    }

    if let Some(skill) = matches.value_of("skill") {
        cmdline.push_line(Line::from_words(&[skill_param[0], skill], 1));
    } else if matches.is_present("warp") {
        cmdline.push_line(Line::from_words(skill_param, 1));
    }

    if matches.is_present("no-monsters") {
        cmdline.push_line(Line::from_word("-nomonsters", 1));
    }

    if matches.is_present("fast") {
        cmdline.push_line(Line::from_word("-fast", 1));
    }

    if matches.is_present("respawn") {
        cmdline.push_line(Line::from_word("-respawn", 1));
    }

    if let Some(passthrough) = matches.values_of("passthrough") {
        for arg in passthrough {
            cmdline.push_line(Line::from_word(arg, 1));
        }
    }

    println!();
    if let Some(render_matches) = matches.value_of("render") {
        let dump_dir = dump_dir()
            .join("Videos")
            .join(iwad_base)
            .join(viddump_folder_name.join(","));
        let renderings = render::collect_renderings(&render_matches, &dump_dir)?;
        batch_render(renderings, &cmdline, dump_dir)?;
    } else {
        println!(
            "Command line: \n'\n{}\n'",
            cmdline.iter_lines().map(|l| l.iter().join(" ")).join("\n")
        );
        if !matches.is_present("no-confirm") {
            Input::<String>::with_theme(&ColorfulTheme {
                prompt_prefix: style("*".into()).yellow(),
                ..Default::default()
            })
            .with_prompt("Press enter to launch Doom.")
            .allow_empty(true)
            .interact()
            .map_err(Error::Io)?;
        }
        run_doom(cmdline.iter_words())?;
    }
    Ok(())
}

fn main() {
    pretty_env_logger::init();
    if let Err(e) = run() {
        error!("{}", e);
        exit(-1);
    }
}
