use std::path::PathBuf;

use clap::StructOpt;
use dialoguer::theme::ColorfulTheme;

mod error;

use error::Error;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
/// Provides shortcuts to the many long-winded options that Doom engines accept.
struct Args {
    #[clap(short, long, value_name = "LEVEL", default_value_t = 21)]
    /// Set the compatibility level to LEVEL. Only relevant for PrBoom-like engines.
    compatibility_level: u8,

    #[clap(short = 'G', long)]
    /// Run Doom under a debugger.
    debug: bool,

    #[clap(
        long,
        env = "DOOM_DIR",
        value_name = "DIR",
        default_value = concat!(env!("HOME"), "/doom")
    )]
    /// Change the location of your Doom configuration.
    doom_dir: PathBuf,

    #[clap(short, long)]
    /// Play the game with ENGINE instead of the first one in the configuration.
    engine: Option<String>,

    #[clap(short = 'x', long, value_name = "WAD", multiple_values = true)]
    /// Add "extra" PWADs to the game.
    ///
    /// The difference between this and the '--pwads' parameter is that extra pwads are not
    /// used when rendering demos to video.
    extra_pwads: Vec<PathBuf>,

    #[clap(short, long)]
    /// Enable fast monsters.
    fast: bool,

    #[clap(short, long)]
    /// Set the screen resolution to 'WxH'.
    ///
    /// Only supported on Boom-derived sourceports, such as PrBoom+.
    geometry: Option<String>,

    #[clap(short, long, value_name = "WAD", default_value = "DOOM2.WAD")]
    /// Set the IWAD to use.
    iwad: PathBuf,

    #[clap(short, long)]
    /// Don't ask for confirmation before running Doom.
    no_confirm: bool,

    #[clap(long)]
    /// Play the game with no monsters.
    no_monsters: bool,

    #[clap(long)]
    /// Attempt to force pistol start on each level.
    ///
    /// Currently only works on Crispy Doom and PrBoom+.
    pistol_start: bool,

    #[clap(short = 'd', long, value_name = "DEMO")]
    /// Play back a DEMO you previously recorded.
    ///
    /// See '-r'.
    play_demo: Option<PathBuf>,

    #[clap(short, long, multiple_values = true, value_name = "WAD")]
    /// Add PWADs to the game.
    pwads: Vec<PathBuf>,

    #[clap(
        short,
        long,
        value_name = "DEMO",
        conflicts_with_all = &["record-from-to", "render", "render-with-name"]
    )]
    /// Record a DEMO.
    ///
    /// The path you provide is relative to $DOOM_DIR/demo.
    record: Option<PathBuf>,

    #[clap(
        long,
        number_of_values = 2,
        value_names = &["FROM", "TO"],
        conflicts_with_all = &["record", "render", "render-with-name"]
    )]
    /// Play back FROM while simultaneously recording to TO.
    ///
    /// This is useful for TAS demos because you may interrupt the playback and begin playing
    /// at any time.
    record_from_to: Option<Vec<PathBuf>>,

    #[clap(
        short = 'R',
        long,
        value_name = "DEMO",
        conflicts_with_all = &["render-with-name", "record", "record-from-to"]
    )]
    /// Render a demo to a video.
    ///
    /// The video's location will be your current working directory (or $DOOM_VIDEO_DIR), with a name matching
    /// the demo's name.
    ///
    /// To override this behavior, pass '--render-with-name' instead.
    render: Option<PathBuf>,

    #[clap(
        long,
        number_of_values = 2,
        value_names = &["DEMO", "VIDEO"],
        conflicts_with_all = &["render", "record", "record-from-to"]
    )]
    /// Render DEMO to VIDEO.
    ///
    /// This is a customizable version of '-R'.
    render_with_name: Option<Vec<PathBuf>>,

    #[clap(long)]
    /// Enable respawning monsters.
    respawn: bool,

    #[clap(long)]
    /// Enable short tics.
    ///
    /// By default, this CLI will pass -longtics when recording demos for convenience.
    short_tics: bool,

    #[clap(
        short,
        long,
        value_name = "NUM",
        validator = |val| match val.parse::<u8>() {
            Ok(i) => {
                if (1..=5).contains(&i) {
                    Ok(())
                } else {
                    Err(String::from("Skills range from 1 to 5."))
                }
            }
            Err(e) => Err(e.to_string())
        },
        default_value_t = 4,
    )]
    /// Set the skill level.
    skill: u8,

    #[clap(short, long)]
    /// Set the video mode.
    ///
    /// This translates to the '-vidmode' parameter for PrBoom-derived ports,
    /// and does nothing in other ports.
    video_mode: Option<String>,

    #[clap(short, long, value_name = "NUM")]
    /// Start the game on the specified level.
    warp: Option<u8>,

    #[clap(multiple_values = true)]
    /// Pass arguments directly to the Doom engine.
    passthrough: Vec<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let args = Args::parse();

    if !args.doom_dir.exists() {
        let answer = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "You don't have a Doom directory at {}. Create it?",
                args.doom_dir.to_string_lossy()
            ))
            .interact()
            .map_err(Error::Io)?;
        if answer {
            std::fs::create_dir_all(&args.doom_dir).map_err(Error::Io)?;
        } else {
            eprintln!("ERROR: Cannot continue. See --help for details on configuring the Doom directory.");
            return Err(Error::NoDoomDir);
        }
    }

    Ok(())
}
