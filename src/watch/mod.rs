mod watcher;
pub use watcher::wait_for_changes;

use crate::build_target;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};

use notify::{self, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub const COOLDOWN_PERIOD: Duration = Duration::from_millis(2000);
const JAVASCRIPT_PATH: &str = "./";

// watch a project for changes and re-build it when necessary,
// outputting a build event to tx.
pub fn watch_and_build(
    target: &Target,
    tx: Option<mpsc::Sender<()>>,
) -> Result<(), failure::Error> {
    let build = target.build.clone();
    let target = target.clone();
    thread::spawn::<_, Result<(), failure::Error>>(move || {
        let (watcher_tx, watcher_rx) = mpsc::channel();
        let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1))?;

        match build {
            None => {
                watcher.watch(JAVASCRIPT_PATH, RecursiveMode::Recursive)?;
                StdOut::info(&format!("watching {:?}", &JAVASCRIPT_PATH));

                loop {
                    match wait_for_changes(&watcher_rx, COOLDOWN_PERIOD) {
                        Ok(_path) => {
                            if let Some(tx) = tx.clone() {
                                tx.send(()).expect("--watch change message failed to send");
                            }
                        }
                        Err(e) => {
                            log::debug!("{:?}", e);
                            StdOut::user_error("Something went wrong while watching.")
                        }
                    }
                }
            }
            Some(config) => {
                config.verify_watch_dir()?;
                watcher.watch(config.watch_dir, notify::RecursiveMode::Recursive)?;

                loop {
                    match wait_for_changes(&watcher_rx, COOLDOWN_PERIOD) {
                        Ok(_path) => match build_target(&target) {
                            Ok(output) => {
                                StdOut::success(&output);
                                if let Some(tx) = tx.clone() {
                                    tx.send(()).expect("--watch change message failed to send");
                                }
                            }
                            Err(e) => StdOut::user_error(&e.to_string()),
                        },
                        Err(e) => {
                            log::debug!("{:?}", e);
                            StdOut::user_error("Something went wrong while watching.")
                        }
                    }
                }
            }
        }
    });

    Ok(())
}
