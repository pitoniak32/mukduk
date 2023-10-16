use anyhow::Result;
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};

use crate::config::ConfigEnvKey;

mod config;

fn main() -> Result<()> {
    let home_dir: PathBuf = PathBuf::from(ConfigEnvKey::Home);
    let config_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGConfig);
    let data_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGData);
    let state_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGState);

    println!("{home_dir:#?}, {config_dir:#?}, {data_dir:#?}, {state_dir:#?}");
    println!("{:#?}", get_directories(config_dir)?);

    Ok(())
}

fn get_directories(path: PathBuf) -> Result<Vec<PathBuf>> {
    Ok(fs::read_dir(path)?
        .filter_map(|dir| match dir {
            Ok(dir) => match dir.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        Some(dir.path())
                    } else {
                        None
                    }
                }
                Err(err) => {
                    println!("An error occured, skipping entry: {err}");
                    None
                }
            },
            Err(err) => {
                println!("An error occured, skipping entry: {err}");
                None
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn should() {
        assert_eq!(true, false)
    }
}
