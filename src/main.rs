use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use inquire::Select;
use std::{fmt::Display, fs, path::PathBuf};

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
    #[clap(subcommand)]
    Project(ProjectSubcommand),
}

#[derive(Subcommand)]
enum ProjectSubcommand {
    Open(ProjectArgs),
}

/// Doc comment
#[derive(Args)]
struct ProjectArgs {
    #[arg(short, long)]
    multiplexer: Multi,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Multi {
    Tmux,
    Zellij,
}

impl Commands {
    fn handle_cmd(command: Commands) -> Result<()> {
        match command {
            Commands::Project(project_cmd) => {
                match project_cmd {
                    ProjectSubcommand::Open(project_open_args) => {
                        let project = pick_project()?;

                        match project_open_args.multiplexer {
                            Multi::Tmux => {
                                log::debug!(
                                    "opening {:?} session with project: {:?}!",
                                    project_open_args.multiplexer,
                                    project
                                )
                                // TODO: implement Command to create tmux session.
                            }
                            Multi::Zellij => {
                                log::debug!(
                                    "opening {:?} session with project: {:?}!",
                                    project_open_args.multiplexer,
                                    project
                                )
                                // TODO: implement Command to create zellij session.
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Project {
    path: PathBuf,
    name: String,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn pick_project() -> Result<Project> {
    let proj_dir: PathBuf = PathBuf::from(ConfigEnvKey::ProjDir);

    log::debug!("Using project_dir: {:?}", &proj_dir);

    let projects = get_directories(proj_dir)?
        .iter()
        .map(|d| Project {
            path: d.to_path_buf(),
            name: d
                .file_name()
                .expect("file_name should be representable as a String")
                .to_string_lossy()
                .to_string(),
        })
        .collect();

    let project = Select::new(&format!("Select your project:"), projects)
        .prompt()
        .unwrap();

    log::debug!("selected: {}", project);

    Ok(project)
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
