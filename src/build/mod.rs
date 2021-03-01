use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdErr};

// Internal build logic, called by both `build` and `publish`
// TODO: return a struct containing optional build info and construct output at command layer
pub fn build_target(target: &Target) -> Result<String, failure::Error> {
    match &target.build {
        None => {
            let msg = "Basic JavaScript project found. Skipping unnecessary build!".to_string();
            Ok(msg)
        }
        Some(config) => {
            if let Some((cmd_str, mut cmd)) = config.build_command() {
                StdErr::working(format!("Running {}", cmd_str).as_ref());
                let build_result = cmd.spawn()?.wait()?;
                if build_result.success() {
                    Ok(String::from("Build completed successfully!"))
                } else if let Some(code) = build_result.code() {
                    Err(failure::err_msg(format!(
                        "Build failed! Status Code: {}",
                        code
                    )))
                } else {
                    Err(failure::err_msg("Build failed."))
                }
            } else {
                Ok(String::from("No build command specified, skipping build."))
            }
        }
    }
}
