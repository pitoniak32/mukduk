use anyhow::Result;
use std::{env, fs, path::PathBuf, process::Command};

use crate::config::ConfigEnvKey;

mod config;

fn main() -> Result<()> {
    // let home_dir: PathBuf = PathBuf::from(ConfigEnvKey::Home);
    // let config_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGConfig);
    // let data_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGData);
    // let state_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGState);
    let proj_dir: PathBuf = PathBuf::from(ConfigEnvKey::ProjDir);
    let args: Vec<String> = env::args().collect();

    if args.get(1).is_some_and(|a| a == "true") {
        let dirs = get_directories(proj_dir)?;
        list_directories(dirs);
    } else {
        let mut user_input = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut user_input)?;
        let user_input = user_input.trim();
        println!("got: {}", user_input);

        let result = Command::new("tmux")
            .arg("new-session")
            .arg("-Ads")
            .arg(user_input)
            .arg("-c")
            .arg("~")
            .spawn()
            .unwrap();
        dbg!(result);
    }

    Ok(())
}

fn list_directories(dirs: Vec<PathBuf>) {
    for dir in dirs {
        println!("{}", dir.file_name().unwrap().to_string_lossy());
    }
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
                    println!("An error occurred, skipping entry: {err}");
                    None
                }
            },
            Err(err) => {
                println!("An error occurred, skipping entry: {err}");
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
