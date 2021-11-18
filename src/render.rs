use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use itertools::Itertools;
use log::info;
use log::warn;

use crate::cmd::CommandLine;
use crate::cmd::Line;
use crate::job::Job;
use crate::search_file;
use crate::Error;
use crate::FileType;

static CANCELLABLE: AtomicBool = AtomicBool::new(false);
static PAUSED: AtomicBool = AtomicBool::new(false);

pub(crate) fn batch_render(
    mut renderings: Vec<Job>,
    cmdline: &CommandLine,
    dump_dir: PathBuf,
) -> Result<(), crate::Error> {
    let (job_sender, job_receiver) = channel::<Result<Job, Error>>();
    let (unpause_sender, unpause_receiver) = channel::<()>();
    ctrlc::set_handler(move || {
        if CANCELLABLE.load(Ordering::Relaxed) {
            PAUSED.store(true, Ordering::SeqCst);
            let extra_demos = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter demo names, separated by spaces: ")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    job_sender.send(Err(Error::Io(e))).unwrap();
                    String::new()
                });

            if extra_demos.split_whitespace().next().is_none() {
                warn!("You didn't enter any demo names.");
                return;
            }
            let jobs_sending_result = extra_demos
                .split_whitespace()
                .map(|d| search_file(d, FileType::Demo))
                .collect::<Result<_, _>>()
                .and_then(|d: Vec<_>| {
                    d.into_iter().flatten().try_for_each(|demo_name| {
                        let name = demo_name
                            .file_stem()
                            .ok_or_else(|| {
                                Error::NoFileStem(demo_name.to_string_lossy().into_owned())
                            })
                            .map(|name| name.to_owned());
                        name.and_then(|name| {
                            let video_name = dump_dir.join({
                                let mut name = name.clone();
                                name.push(".mp4");
                                name
                            });
                            job_sender
                                .send(
                                    name.to_str()
                                        .ok_or_else(|| {
                                            Error::NonUtf8Path(
                                                name.as_os_str().to_string_lossy().into_owned(),
                                            )
                                        })
                                        .map(|name| Job {
                                            name: name.to_owned(),
                                            demo_name: demo_name.clone(),
                                            video_name,
                                        }),
                                )
                                .map_err(|e| Error::Send(Box::new(e)))
                        })
                    })
                });

            if jobs_sending_result.is_err() {
                job_sender
                    .send(jobs_sending_result.map(|_| Job {
                        name: String::new(),
                        demo_name: PathBuf::new(),
                        video_name: PathBuf::new(),
                    }))
                    .unwrap_or_else(|e| job_sender.send(Err(Error::Send(Box::new(e)))).unwrap());
            }

            PAUSED.store(false, Ordering::SeqCst);
            unpause_sender.send(()).unwrap();
        } else {
            println!();
            println!("Received interrupt, exiting. Goodbye.");
            exit(0);
        }
    })
    .map_err(Error::SignalHandler)?;
    let mut i = 1;
    while !renderings.is_empty() {
        info!("====== RENDERING QUEUE ======");
        for job in &renderings {
            info!(
                "{}  ==>  {}",
                job.demo_name.to_str().ok_or_else(|| Error::NonUtf8Path(
                    job.demo_name.to_string_lossy().into_owned()
                ))?,
                job.name
            );
        }
        info!("==== END RENDERING QUEUE ====");

        let job = renderings.remove(0);
        let render_cmdline = {
            let mut rcmdline = cmdline.clone();
            rcmdline.push_line(Line::from_word("-timedemo", 1));
            rcmdline.push_line(Line::from_word(
                job.demo_name.to_str().ok_or_else(|| {
                    Error::NonUtf8Path(job.demo_name.to_string_lossy().into_owned())
                })?,
                2,
            ));

            rcmdline.push_line(Line::from_word("-viddump", 1));
            rcmdline.push_line(Line::from_word(
                job.video_name.to_str().ok_or_else(|| {
                    Error::NonUtf8Path(job.video_name.to_string_lossy().into_owned())
                })?,
                2,
            ));
            rcmdline
        };
        println!(
            "Command line #{}: \n'\n{}\n'",
            i,
            render_cmdline
                .iter_lines()
                .map(|l| l.iter().join(" "))
                .join("\n")
        );
        if i == 1 {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Press enter to begin {}rendering.",
                    if !renderings.is_empty() { "batch" } else { "" }
                ))
                .allow_empty(true)
                .interact()
                .map_err(Error::Io)?;
        } else {
            CANCELLABLE.store(true, Ordering::SeqCst);
            info!("Continuing batch rendering in 10 seconds. Press <C-c> to add more demos to the queue.");
            sleep(Duration::from_secs(10));
            if PAUSED.load(Ordering::SeqCst) {
                unpause_receiver.recv()?;
            }
            CANCELLABLE.store(false, Ordering::SeqCst);
            for job in job_receiver.try_iter() {
                renderings.push(job?);
            }
        }

        crate::run_doom(render_cmdline.iter_words())?;

        i += 1;
    }
    Ok(())
}
