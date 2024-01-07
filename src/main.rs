use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

use config::ConfigEnvKey;
use fzf::FzfCmd;
use git_lib::repo::GitRepo;
use helper::{fzf_get_sessions, get_project};
use multiplexer::{Multiplexer, Multiplexers};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, File},
    path::PathBuf,
    process,
};

mod config;
mod helper;
mod multiplexer;

mod fzf;
mod tmux;
mod zellij;

fn main() -> Result<()> {
    let cli = MukdukCli::init()?;

    cli.handle_cmd()?;

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about)]
/// Manage your terminal environment.
struct MukdukCli {
    #[clap(skip)]
    context: MukdukContext,

    #[clap(flatten)]
    args: SharedArgs,

    #[command(subcommand)]
    command: Option<MukdukCommands>,
}

impl MukdukCli {
    fn init() -> Result<Self> {
        // let home_dir: PathBuf = PathBuf::from(ConfigEnvKey::Home);
        // let config_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGConfig);
        // let data_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGData);
        // let state_dir: PathBuf = PathBuf::from(ConfigEnvKey::XDGState);
        let mut cli = MukdukCli::parse();
        env_logger::builder()
            .filter_level(cli.args.verbosity.log_level_filter())
            .parse_default_env()
            .init();

        cli.set_config_path()?;
        cli.read_config()?;

        log::debug!("{:#?}", &cli.args);
        log::debug!("{:#?}", &cli.context);

        Ok(cli)
    }

    fn set_config_path(&mut self) -> Result<()> {
        if let Some(config_path) = &self.args.config_path {
            if let Ok(curr) = std::fs::canonicalize(config_path) {
                log::debug!("checking {}", curr.to_string_lossy());
                if !curr.exists() {
                    eprintln!(
                        "\n{}\n",
                        "Provided config path does not exist.".red().bold()
                    );
                    process::exit(1);
                }
                self.args.config_path = Some(curr.clone());
                self.context.config_path = curr.clone();
            }
        } else {
            let mut path = PathBuf::from(ConfigEnvKey::XDGConfig);
            if path.exists() {
                path.push("mukduk");
                if !path.exists() {
                    fs::create_dir(&path)?;
                }
                path.push("config.yml");
                if !path.exists() {
                    File::create(&path)?;
                }
            } else {
                let mut path = PathBuf::from(ConfigEnvKey::Home);
                if path.exists() {
                    path.push(".mukdukrc.yml");
                    if !path.exists() {
                        File::create(&path)?;
                    }
                }
            }
            self.args.config_path = Some(path.clone());
            self.context.config_path = path.clone();
        }
        Ok(())
    }

    fn read_config(&mut self) -> Result<()> {
        let config_path = &self.context.config_path;
        log::trace!("loading config from {}...", config_path.to_string_lossy());
        self.context.config = serde_yaml::from_str(&fs::read_to_string(config_path)?)?;
        log::trace!("config: {:#?}", self.context.config);
        log::trace!("config loaded!");
        Ok(())
    }
}

#[derive(Args, Debug)]
struct SharedArgs {
    #[arg(long, env)]
    projects_dir: PathBuf,

    /// Override '$XDG_CONFIG_HOME/config.yml' or '$HOME/.mukdukrc.yml' defaults.
    #[arg(short, long)]
    config_path: Option<PathBuf>,

    /// Allow interactive choice of project dirs listed in config file.
    #[arg(short, long)]
    pick_projects_dir: bool,

    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct MukdukContext {
    config_path: PathBuf,
    config: MukdukConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct MukdukConfig {
    projects: Option<Vec<PathBuf>>,
}

#[derive(Subcommand)]
enum MukdukCommands {
    #[clap(subcommand)]
    /// Commands for managing projects.
    Project(ProjectSubcommand),
}

#[derive(Subcommand)]
enum ProjectSubcommand {
    /// Open a session.
    Open {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Open a scratch session. defaults: (name = scratch, path = $HOME)
    Scratch {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Kill sessions.
    Kill {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Open new unique session in $HOME and increment prefix (available: 0-9).
    Home {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        #[clap(flatten)]
        sess_args: SessionArgs,
    },
    /// Clone a new repo into your projects dir.
    New {
        #[clap(flatten)]
        proj_args: ProjectArgs,
        ssh_uri: String,
    }, // Like ThePrimagen Harpoon in nvim but for multiplexer sessions
       // Harpoon(ProjectArgs),
}

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    /// Which multiplexer session should be created.
    pub multiplexer: Multiplexers,
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub name: Option<String>,

    #[arg(short, long)]
    /// Name of session, defaults to project_dir name
    pub project_dir: Option<PathBuf>,
}

impl MukdukCli {
    fn handle_cmd(self) -> Result<()> {
        if let Some(cmd) = self.command {
            let mut projects_dir = self.args.projects_dir;
            if self.args.pick_projects_dir {
                log::trace!("user picking project dir...");
                if let Some(dirs) = self.context.config.projects {
                    let string_dir_names: Vec<String> = dirs
                        .iter()
                        .map(|d| d.to_string_lossy().to_string())
                        .collect();
                    let selected = PathBuf::from(FzfCmd::new().find_vec(string_dir_names)?);
                    log::trace!(
                        "expanding project dir selection: [{}]",
                        selected.to_string_lossy()
                    );
                    match std::fs::canonicalize(selected) {
                        Ok(curr) => {
                            log::trace!(
                                "user picked [{}] as project dir.",
                                projects_dir.to_string_lossy()
                            );
                            projects_dir = curr
                        }
                        Err(err) => {
                            log::trace!("failed expanding project dir selection. using default of [{}]: {err}", projects_dir.to_string_lossy());
                        }
                    }
                }
            }
            MukdukCommands::handle_cmd(cmd, projects_dir)?;
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

impl MukdukCommands {
    fn handle_cmd(mukduk_command: MukdukCommands, projects_dir: PathBuf) -> Result<()> {
        match mukduk_command {
            MukdukCommands::Project(project_sub_cmd) => {
                ProjectSubcommand::handle_cmd(project_sub_cmd, projects_dir)
            }
        }
    }
}

impl ProjectSubcommand {
    fn handle_cmd(project_sub_cmd: ProjectSubcommand, projects_dir: PathBuf) -> Result<()> {
        match project_sub_cmd {
            ProjectSubcommand::Open {
                proj_args,
                sess_args,
            } => {
                let project =
                    get_project(projects_dir, &proj_args.project_dir, proj_args.name.clone())?;
                sess_args.multiplexer.open(&proj_args, project)?;
                Ok(())
            }
            ProjectSubcommand::Scratch {
                proj_args,
                sess_args,
            } => {
                sess_args.multiplexer.open(
                    &proj_args,
                    Project::new(
                        proj_args
                            .project_dir
                            .clone()
                            .unwrap_or(PathBuf::from(ConfigEnvKey::Home)),
                        proj_args.name.clone().unwrap_or("scratch".to_string()),
                    ),
                )?;
                Ok(())
            }
            ProjectSubcommand::Kill {
                proj_args: _,
                sess_args,
            } => {
                let sessions = sess_args.multiplexer.get_sessions();
                log::debug!("sessions: {sessions:?}");
                let picked_sessions = fzf_get_sessions(sessions)?;
                sess_args.multiplexer.kill_sessions(picked_sessions)?;
                Ok(())
            }
            ProjectSubcommand::Home {
                proj_args: _,
                sess_args,
            } => sess_args.multiplexer.unique_session(),
            ProjectSubcommand::New {
                proj_args: _,
                ssh_uri,
            } => {
                log::debug!("Attempting to clone {ssh_uri}...");
                let results = GitRepo::from_ssh_uri_multi(&[&ssh_uri], &projects_dir);
                for result in results {
                    if let Err(err) = result {
                        log::error!("Failed cloning with: {err:?}");
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    path: PathBuf,
    name: String,
}

impl Project {
    pub fn new(path: PathBuf, name: String) -> Self {
        Project {
            path,
            name: name.replace('.', "_"),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::Project;

    #[test]
    fn should_update_project_name_with_underscores() {
        assert_eq!(
            Project::new(PathBuf::from(""), ".test.test".to_string()).get_name(),
            "_test_test".to_string()
        )
    }
}
