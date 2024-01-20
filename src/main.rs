use anyhow::Result;
use clap::{Args, Parser};
use colored::Colorize;

use commands::MukdukCommands;
use config::ConfigEnvKey;
use fzf::FzfCmd;
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

mod commands;

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
        let mut cli = Self::parse();
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
                self.context.config_path = curr;
            }
        } else {
            let mut path = PathBuf::try_from(ConfigEnvKey::XDGConfig)?;
            if path.exists() {
                path.push("mukduk");
                if !path.exists() {
                    fs::create_dir(&path)?;
                }
                path.push("config.toml");
                if !path.exists() {
                    File::create(&path)?;
                }
            } else {
                let mut path = PathBuf::try_from(ConfigEnvKey::Home)?;
                if path.exists() {
                    path.push(".mukdukrc.toml");
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
        self.context.config = toml::from_str(&fs::read_to_string(config_path)?)?;
        log::trace!("config: {:#?}", self.context.config);
        log::trace!("config loaded!");
        Ok(())
    }
}

#[derive(Args, Debug)]
struct SharedArgs {
    #[arg(long, env)]
    projects_dir: Option<PathBuf>,

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
    projects_dir: ProjectsDir,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ProjectsDir {
    default: Option<PathBuf>,
    options: Option<Vec<PathBuf>>,
}

impl MukdukCli {
    fn handle_cmd(self) -> Result<()> {
        if let Some(cmd) = self.command {
            let mut projects_dir = self
                .args
                .projects_dir
                .or(self.context.config.projects_dir.default)
                .expect("should be set");
            if self.args.pick_projects_dir {
                log::trace!("user picking project dir...");
                if let Some(dirs) = self.context.config.projects_dir.options {
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

#[derive(Debug, Clone)]
pub struct Project {
    path: PathBuf,
    name: String,
}

impl Project {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
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
