use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use inquire::Select;
use multiplexer::{Multiplexer, Multiplexers};
use std::{fmt::Display, fs, path::PathBuf};

use crate::config::ConfigEnvKey;

mod config;
mod multiplexer;

mod tmux;
mod zellij;

#[derive(Parser)]
#[command(author, version, about)]
/// Manage your terminal environment.
struct MukdukCli {
    #[arg(short, long)]
    projects_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<MukdukCommands>,

    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

impl MukdukCli {
    fn handle_cmd(self) -> Result<()> {
        if let Some(cmd) = self.command {
            MukdukCommands::handle_cmd(cmd, self.projects_dir)?;
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
enum MukdukCommands {
    #[clap(subcommand)]
    Project(ProjectSubcommand),
}

#[derive(Subcommand)]
enum ProjectSubcommand {
    Create(ProjectArgs),
    Open(ProjectArgs),
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be created.
    pub multiplexer: Multiplexers,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub name: Option<String>,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub project_dir: Option<PathBuf>,
}

impl ProjectSubcommand {
    fn handle_cmd(project_sub_cmd: ProjectSubcommand, projects_dir: Option<PathBuf>) -> Result<()> {
        match project_sub_cmd {
            ProjectSubcommand::Create(proj_args) => {
                let project =
                    get_project(projects_dir, &proj_args.project_dir, proj_args.name.clone())?;
                ProjectSubcommand::handle_create_cmd(&proj_args, project)?;
                Ok(())
            }
            ProjectSubcommand::Open(proj_args) => {
                let project =
                    get_project(projects_dir, &proj_args.project_dir, proj_args.name.clone())?;
                ProjectSubcommand::handle_open_cmd(&proj_args, project)?;
                Ok(())
            }
        }
    }

    fn handle_create_cmd(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        proj_args.multiplexer.create(proj_args, project)?;
        Ok(())
    }

    fn handle_open_cmd(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        proj_args.multiplexer.open(proj_args, project)?;
        Ok(())
    }
}

impl MukdukCommands {
    fn handle_cmd(mukduk_command: MukdukCommands, projects_dir: Option<PathBuf>) -> Result<()> {
        match mukduk_command {
            MukdukCommands::Project(project_sub_cmd) => {
                ProjectSubcommand::handle_cmd(project_sub_cmd, projects_dir)
            }
        }
    }
}

#[derive(Debug)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn get_project(
    projects_dir: Option<PathBuf>,
    project_dir: &Option<PathBuf>,
    name: Option<String>,
) -> Result<Project> {
    let project: Project;
    if let Some(selected_project) = project_dir {
        project = Project {
            name: name.unwrap_or(
                selected_project
                    .file_name()
                    .expect("file_name should be representable as a String.")
                    .to_string_lossy()
                    .to_string(),
            ),
            path: selected_project.clone(),
        }
    } else {
        project = pick_project(projects_dir)?;
    }

    Ok(project)
}

fn pick_project(projects_dir: Option<PathBuf>) -> Result<Project> {
    let proj_dir: PathBuf = projects_dir.unwrap_or(PathBuf::from(ConfigEnvKey::ProjDir));

    log::info!("Using project_dir: {:?}", &proj_dir);

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

    let project = Select::new("Select your project:", projects)
        .prompt()
        .unwrap();

    log::info!("selected: {}", project);

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
                    eprintln!("An error occurred, skipping entry: {err}");
                    None
                }
            },
            Err(err) => {
                eprintln!("An error occurred, skipping entry: {err}");
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
