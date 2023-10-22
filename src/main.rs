use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;

use config::ConfigEnvKey;
use helper::{fzf_get_sessions, get_project};
use multiplexer::{Multiplexer, Multiplexers};
use std::{fmt::Display, path::PathBuf};

mod config;
mod helper;
mod multiplexer;

mod tmux;
mod zellij;

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

#[derive(Subcommand)]
enum MukdukCommands {
    #[clap(subcommand)]
    /// Commands for dealing with projects.
    Project(ProjectSubcommand),
}

#[derive(Subcommand)]
enum ProjectSubcommand {
    /// Open a session.
    Open(ProjectArgs),
    /// Open a scratch session. defaults: (name = scratch, path = $HOME)
    Scratch(ProjectArgs),
    /// Kill sessions.
    Kill(ProjectArgs),
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

impl MukdukCommands {
    fn handle_cmd(mukduk_command: MukdukCommands, projects_dir: Option<PathBuf>) -> Result<()> {
        match mukduk_command {
            MukdukCommands::Project(project_sub_cmd) => {
                ProjectSubcommand::handle_cmd(project_sub_cmd, projects_dir)
            }
        }
    }
}

impl ProjectSubcommand {
    fn handle_cmd(project_sub_cmd: ProjectSubcommand, projects_dir: Option<PathBuf>) -> Result<()> {
        match project_sub_cmd {
            ProjectSubcommand::Open(proj_args) => {
                let project =
                    get_project(projects_dir, &proj_args.project_dir, proj_args.name.clone())?;
                proj_args.multiplexer.open(&proj_args, project)?;
                Ok(())
            }
            ProjectSubcommand::Scratch(proj_args) => {
                proj_args.multiplexer.open(
                    &proj_args,
                    Project {
                        name: proj_args.name.clone().unwrap_or("scratch".to_string()),
                        path: proj_args
                            .project_dir
                            .clone()
                            .unwrap_or(PathBuf::from(ConfigEnvKey::Home)),
                    },
                )?;
                Ok(())
            }
            ProjectSubcommand::Kill(proj_args) => {
                let sessions = proj_args.multiplexer.get_sessions();
                let picked_sessions = fzf_get_sessions(sessions)?;
                proj_args.multiplexer.kill_sessions(picked_sessions)?;
                Ok(())
            }
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn should() {
        assert_eq!(true, false)
    }
}
