use clap::Subcommand;
use std::path::PathBuf;

use self::project::ProjectSubcommand;
pub mod project;

#[derive(Subcommand, Debug)]
pub enum MukdukCommands {
    #[clap(subcommand)]
    /// Commands for managing projects.
    Project(ProjectSubcommand),
}

impl MukdukCommands {
    pub fn handle_cmd(mukduk_command: Self, projects_dir: PathBuf) -> anyhow::Result<()> {
        match mukduk_command {
            Self::Project(project_sub_cmd) => {
                ProjectSubcommand::handle_cmd(project_sub_cmd, projects_dir)
            }
        }
    }
}
