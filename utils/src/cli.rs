use anyhow::Result;
use std::path::{Path, PathBuf};

/*
    get config for file
*/
pub fn get_config_file(s: &str) -> Result<PathBuf> {
    let path = Path::new(s);
    if path.exists() {
        Ok(path.to_path_buf())
    } else {
        Err(anyhow::anyhow!("config file not found"))
    }
}

pub fn get_default_config(name: &str) -> Result<PathBuf> {
    let paths = [
        format!("{}/.config/{}", std::env::var("HOME").unwrap(), name),
        format!("{}/config/{}", std::env::var("PWD").unwrap(), name),
        format!("./{}", name),
        format!("/etc/{}", name),
    ];

    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(Path::new(path).to_path_buf());
        }
    }

    Err(anyhow::anyhow!("Config file not found. You can either specify it with the --config option or put it in one of the following locations: {}", paths.join(", ")))
}
