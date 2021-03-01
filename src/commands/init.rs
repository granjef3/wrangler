use std::path::{Path, PathBuf};

use crate::commands::validate_worker_name;
use crate::settings::toml::{Manifest, Site};
use crate::terminal::message::{Message, StdOut};
pub fn init(name: Option<&str>, site_flag: bool) -> Result<(), failure::Error> {
    if Path::new("./wrangler.toml").exists() {
        if site_flag {
            let msg = r#"A wrangler.toml file already exists!

To add Workers Sites to your existing wrangler.toml, please add this section:

[site]
bucket = "" # this should point to the directory with static assets
entry-point = "workers-site"

"#;
            failure::bail!(msg);
        } else {
            failure::bail!("A wrangler.toml file already exists! Please remove it before running this command again.");
        }
    }
    let dirname = get_current_dirname()?;
    let name = name.unwrap_or_else(|| &dirname);
    validate_worker_name(name)?;

    let config_path = PathBuf::from("./");

    if site_flag {
        let site = Site::default();
        Manifest::generate(name.to_string(), &config_path, Some(site.clone()))?;

        site.scaffold_worker()?;
        StdOut::success("Successfully scaffolded workers site");
    } else {
        Manifest::generate(name.to_string(), &config_path, None)?;
    }

    StdOut::success("Succesfully created a `wrangler.toml`");
    Ok(())
}

fn get_current_dirname() -> Result<String, failure::Error> {
    let current_path = std::env::current_dir()?;
    let parent = current_path.parent();
    let dirname = match parent {
        Some(parent) => current_path.strip_prefix(parent)?.display().to_string(),
        None => "worker".to_string(),
    };
    Ok(dirname)
}
