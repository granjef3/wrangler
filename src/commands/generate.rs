use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use failure::ResultExt;

use crate::commands;
use crate::commands::validate_worker_name;
use crate::settings::toml::{Manifest, Site};

pub fn generate(name: &str, template: &str, site: bool) -> Result<(), failure::Error> {
    validate_worker_name(name)?;

    let dirname_exists = match directory_exists(name) {
        Ok(val) => val,
        Err(_) => true,
    };

    let new_name = if dirname_exists {
        match generate_name(name) {
            Ok(val) => val,
            Err(_) => {
                log::debug!(
                    "Failed to auto-increment name for a new worker project, using '{}'",
                    name
                );
                String::from(name)
            }
        }
    } else {
        String::from(name)
    };

    log::info!("Generating a new worker project with name '{}'", new_name);
    clone_repo(&new_name, template)?;

    update_package_json_name_if_exists(&new_name);

    let config_path = PathBuf::from("./").join(&name);
    // TODO: this is tightly coupled to our site template. Need to remove once
    // we refine our generate logic.
    let generated_site = if site {
        Some(Site::new("./public"))
    } else {
        None
    };
    Manifest::generate(new_name, &config_path, generated_site)?;

    Ok(())
}

pub fn clone_repo(name: &str, template: &str) -> Result<(), failure::Error> {
    let args = ["clone", "--depth", "1", template, name];

    let command = command("git", &args);
    let command_name = format!("{:?}", command);
    commands::run(command, &command_name)?;

    // TODOi(now): Check for liquid template statements and warn the user their template should be updated.

    fs::remove_dir_all(env::current_dir()?.join(name).join(".git"))
        .context("Error cleaning up cloned template")?;
    Ok(())
}

fn update_package_json_name_if_exists(name: &str) {
    // This is a convenience optimization. If it fails, it should not error.
    // TODO: it'd be nice to have an early_return macro for this

    let current_dir = match env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(_) => return,
    };

    let package_json_path = current_dir.join(name).join("package.json");

    if package_json_path.is_file() {
        let package_json_str = match fs::read_to_string(&package_json_path) {
            Ok(s) => s,
            Err(_) => return
        };

        let mut package_json = match package_json_str.parse::<serde_json::Value>() {
            Ok(value) => value,
            Err(_) => return,
        };
        package_json["name"] = serde_json::Value::from(name);

        if let Ok(new_json) = serde_json::to_string_pretty(&package_json) {
            match fs::write(&package_json_path, new_json) {
                Ok(()) => (),
                Err(_) => (),
            };
        };
    }
}

fn generate_name(name: &str) -> Result<String, failure::Error> {
    let mut num = 1;
    let entry_names = read_current_dir()?;
    let mut new_name = construct_name(&name, num);

    while entry_names.contains(&OsString::from(&new_name)) {
        num += 1;
        new_name = construct_name(&name, num);
    }
    Ok(new_name)
}

fn read_current_dir() -> Result<Vec<OsString>, failure::Error> {
    let current_dir = env::current_dir()?;
    let mut entry_names = Vec::new();

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            entry_names.push(entry.file_name());
        }
    }
    Ok(entry_names)
}

fn directory_exists(dirname: &str) -> Result<bool, failure::Error> {
    let entry_names = read_current_dir()?;
    Ok(entry_names.contains(&OsString::from(dirname)))
}

fn construct_name(name: &str, num: i32) -> String {
    format!("{}-{}", name, num)
}

fn command(binary: &str, args: &[&str]) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary);
        c
    } else {
        Command::new(binary)
    };

    c.args(args);
    c
}
