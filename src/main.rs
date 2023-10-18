use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use inquire::Select;
use std::{
    env,
    fmt::Display,
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use crate::config::ConfigEnvKey;

mod config;

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
    Create(ProjectCreateArgs),
    Open(ProjectOpenArgs),
}

#[derive(Args, Debug)]
struct ProjectCreateArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be created.
    multiplexer: Multiplexer,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    name: Option<String>,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    project_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ProjectOpenArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be opened.
    multiplexer: Multiplexer,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    name: Option<String>,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    project_dir: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Multiplexer {
    Tmux,
    Zellij,
}

fn tmux_has_session(project_name: &str) -> bool {
    match Command::new("tmux")
        .args(["has-session", "-t", &format!("={}", project_name)])
        .status()
    {
        Ok(status) => {
            if status.success() {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn not_in_tmux() -> bool {
    match env::var("TMUX") {
        Ok(_) => false,
        Err(_) => true,
    }
}

impl ProjectSubcommand {
    fn handle_cmd(project_sub_cmd: ProjectSubcommand, projects_dir: Option<PathBuf>) -> Result<()> {
        match project_sub_cmd {
            ProjectSubcommand::Create(create_args) => {
                let project = get_project(
                    projects_dir,
                    &create_args.project_dir,
                    create_args.name.clone(),
                )?;
                ProjectSubcommand::handle_create_cmd(&create_args, project)?;
                Ok(())
            }
            ProjectSubcommand::Open(open_args) => {
                let project =
                    get_project(projects_dir, &open_args.project_dir, open_args.name.clone())?;
                ProjectSubcommand::handle_open_cmd(&open_args, project)?;
                Ok(())
            }
        }
    }

    fn handle_create_cmd(create_args: &ProjectCreateArgs, project: Project) -> Result<()> {
        match create_args.multiplexer {
            Multiplexer::Tmux => {
                log::info!(
                    "creating {:?} session with project: {:?}!",
                    create_args.multiplexer,
                    project
                );

                // TODO: implement Command to create tmux session.
                let output = Command::new("tmux")
                    .args([
                        "new-session",
                        "-Ad",
                        "-s",
                        &project.name,
                        "-c",
                        project.path.to_str().unwrap_or_default(),
                    ])
                    .status()?;

                if output.success() {
                    println!(
                        "Session '{}' has been created in '{}'.",
                        project.name,
                        project.path.to_string_lossy()
                    );
                } else {
                    eprintln!("Session failed to be created with exit_code: {}", output);
                }
            }
            Multiplexer::Zellij => {
                log::info!(
                    "creating {:?} session with project: {:?}!",
                    create_args.multiplexer,
                    project
                );
                todo!("not implemented");
                // TODO: implement Command to create zellij session.
            }
        }
        Ok(())
    }

    fn handle_open_cmd(open_args: &ProjectOpenArgs, project: Project) -> Result<()> {
        match open_args.multiplexer {
            Multiplexer::Tmux => {
                log::info!(
                    "opening {:?} session with project: {:?}!",
                    open_args.multiplexer,
                    project
                );

                let output: ExitStatus;
                if not_in_tmux() {
                    let output = Command::new("tmux")
                        .args([
                            "new-session",
                            "-Ad",
                            "-s",
                            &project.name,
                            "-c",
                            project.path.to_str().unwrap_or_default(),
                        ])
                        .status()?;
                } else {
                    if tmux_has_session(&project.name) {
                        println!("Session '{}' already exists, opening.", project.name);
                        let _child = Command::new("tmux")
                            .args(["switch-client", "-t", &project.name])
                            .status()?;
                    } else {
                        println!(
                            "Session '{}' does not already exist, creating and opening.",
                            project.name
                        );
                        output = Command::new("tmux")
                            .args([
                                "new-session",
                                "-A",
                                "-s",
                                &project.name,
                                "-c",
                                project.path.to_str().unwrap_or_default(),
                            ])
                            .status()?;
                        if output.success() {
                            println!("Session '{}' has been opened.", project.name);
                        } else {
                            eprintln!("Session failed to be opened with exit_code: {}", output);
                        }
                    }
                }
            }
            Multiplexer::Zellij => {
                log::info!(
                    "opening {:?} session with project: {:?}!",
                    open_args.multiplexer,
                    project
                )
                // TODO: implement Command to open zellij session.
            }
        }
        Ok(())
    }
}

impl MukdukCommands {
    fn handle_cmd(mukduk_command: MukdukCommands, projects_dir: Option<PathBuf>) -> Result<()> {
        match mukduk_command {
            MukdukCommands::Project(project_cmd) => {
                ProjectSubcommand::handle_cmd(project_cmd, projects_dir)?;
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

    let project = Select::new(&format!("Select your project:"), projects)
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
