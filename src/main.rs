use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

use self_update::cargo_crate_version;

use multiplexer::{Multiplexer, Multiplexers};
use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::config::ConfigEnvKey;

mod config;
mod helper;
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

    Update,
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
            },
            MukdukCommands::Update => update(),
        }
    }
}

#[derive(Debug, Clone)]
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

    let projects: Vec<_> = get_directories(proj_dir)?
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
    let project_name = fzf_get_project_name(
        &projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;

    if project_name.is_empty() {
        eprintln!("\n{}\n", "No project was selected.".red().bold());
        std::process::exit(1);
    }
    Ok(projects.iter().find(|p| p.name == project_name).expect("This should never be None since the project_names list only contains names from the list of projects. If the user does not choose from the list the program will exit.").clone())
}

fn fzf_get_project_name(project_names: &str) -> Result<String> {
    let echo_child = Command::new("echo")
        .arg(project_names)
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(echo_stdout) = echo_child.stdout {
        let fzf_child = Command::new("fzf")
            .stdin(echo_stdout)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?;
        let selected_name = String::from_utf8_lossy(&fzf_child.stdout)
            .trim()
            .to_string();
        return Ok(selected_name);
    }
    Ok("".to_string())
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

fn update() -> Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("pitoniak32")
        .repo_name("mukduk")
        .bin_name("mukduk")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
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
