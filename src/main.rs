use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use inquire::Select;
use std::{env, fmt::Display, fs, path::PathBuf, process::Command};

use crate::config::ConfigEnvKey;

mod config;

#[derive(Parser)]
#[command(author, version, about)]
/// Hi this is the short description.
///
/// This is the longer more details description of what this cli is used for.
struct MukdukCli {
    name: Option<String>,

    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl MukdukCli {
    fn handle_cmd(self) -> Result<()> {
        if let Some(cmd) = self.command {
            Commands::handle_cmd(cmd)?;
        } else {
            eprintln!(
                "\n{}\n",
                "No command was provided! To see commands use `--help`."
                    .yellow()
                    .bold()
            );
            std::process::exit(1);
        }

        Ok(())
    }
}

#[derive(Subcommand)]
enum Commands {
    Project,
}

impl Commands {
    fn handle_cmd(command: Commands) -> Result<()> {
        match command {
            Commands::Project => {
                let proj_dir: PathBuf = PathBuf::from(ConfigEnvKey::ProjDir);
                log::debug!("{:?}", &proj_dir);
                let dirs = get_directories(proj_dir)?;

                let dir = Select::new(
                    &format!("Select your project:"),
                    dirs.iter().map(|d| d.to_string_lossy()).collect(),
                )
                .prompt()
                .unwrap();
                log::info!("selected: {}", dir);
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    // let home_dir: PathBuf = PathBuf::from(ConfigEnvKey::Home);
    // let config_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGConfig);
    // let data_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGData);
    // let state_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGState);
    let cli = MukdukCli::parse();

    env_logger::builder()
        .filter_level(cli.verbosity.log_level_filter())
        .parse_default_env()
        .init();

    cli.handle_cmd()?;

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
