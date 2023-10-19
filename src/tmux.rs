use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    process::Command,
};

use crate::{helper::wrap_command, Project, ProjectArgs};

pub struct Tmux;

impl Tmux {
    pub fn create(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "creating {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );

        let output = wrap_command(Command::new("tmux").args([
            "new-session",
            "-Ad",
            "-s",
            &project.name,
            "-c",
            project.path.to_str().unwrap_or_default(),
        ]))?;

        if output.status.success() {
            log::info!(
                "Session '{}' has been created in '{}'.",
                project.name,
                project.path.to_string_lossy()
            );
        } else {
            eprintln!("{}", "Session failed to be created.".red().bold());
        }

        Ok(())
    }

    pub fn open(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "opening {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );
        if Tmux::not_in() {
            wrap_command(Command::new("tmux").args([
                "new-session",
                "-A",
                "-s",
                &project.name,
                "-c",
                project.path.to_str().unwrap_or_default(),
            ]))?;
        } else if Tmux::has_session(&project.name) {
            log::info!("Session '{}' already exists, opening.", project.name);
            wrap_command(Command::new("tmux").args(["switch-client", "-t", &project.name]))?;
        } else {
            log::info!(
                "Session '{}' does not already exist, creating and opening.",
                project.name
            );

            let output_tmux = wrap_command(Command::new("tmux").args([
                "new-session",
                "-d",
                "-s",
                &project.name,
                "-c",
                project.path.to_str().unwrap_or_default(),
            ]));

            if output_tmux.is_ok_and(|o| o.status.success()) {
                wrap_command(Command::new("tmux").args(["switch-client", "-t", &project.name]))?;
            } else {
                eprintln!("{}", "Session failed to open.".red().bold());
            }
        }

        Ok(())
    }
}

impl Tmux {
    fn has_session(project_name: &str) -> bool {
        let output = wrap_command(Command::new("tmux").args([
            "has-session",
            "-t",
            &format!("={}", project_name),
        ]));

        output.is_ok_and(|o| o.status.success())
    }

    fn not_in() -> bool {
        env::var("TMUX").is_err()
    }
}
