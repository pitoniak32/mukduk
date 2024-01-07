use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::{config::ConfigEnvKey, helper::wrap_command, Project, ProjectArgs};

pub struct Tmux;

impl Tmux {
    pub fn open(_proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "Attempting to open Tmux session with project: {:?}!",
            project,
        );

        if !Tmux::in_session() {
            Tmux::create_new_attached_attach_if_exists(&project.get_name(), &project.get_path())?;
        } else if Tmux::has_session(&project.get_name()) {
            log::info!("Session '{}' already exists, opening.", project.get_name());
            Tmux::switch(&project.get_name())?;
        } else {
            log::info!(
                "Session '{}' does not already exist, creating and opening.",
                project.get_name(),
            );

            if Tmux::create_new_detached(&project.get_name(), &project.get_path())
                .is_ok_and(|o| o.status.success())
            {
                Tmux::switch(&project.get_name())?;
            } else {
                eprintln!("{}", "Session failed to open.".red().bold());
            }
        }

        Ok(())
    }

    pub fn list_sessions() -> Vec<String> {
        String::from_utf8_lossy(
            &wrap_command(Command::new("tmux").arg("ls"))
                .expect("tmux should be able to list sessions")
                .stdout,
        )
        .trim_end()
        .split('\n')
        .map(|s| s.split(':').collect::<Vec<_>>()[0].to_string())
        .filter(|s| !s.is_empty())
        .collect()
    }

    pub fn kill_sessions(sessions: &[String]) -> Result<()> {
        sessions.iter().for_each(|s| {
            if Tmux::kill_session(s).is_ok() {
                if s.is_empty() {
                    log::warn!("No session picked");
                } else {
                    log::info!("Killed {}.", s);
                }
            } else {
                log::error!("Error while killing {}.", s)
            }
        });
        Ok(())
    }

    pub fn unique_session() -> Result<()> {
        for i in 0..10 {
            let name = &i.to_string();
            if !Tmux::has_session(name) {
                if Tmux::create_new_detached(name, &PathBuf::from(ConfigEnvKey::Home))
                    .is_ok_and(|o| o.status.success())
                {
                    Tmux::switch(name)?;
                    break;
                } else {
                    eprintln!("{}", "Session failed to open.".red().bold());
                }
            }
        }
        Ok(())
    }
}

impl Tmux {
    #[allow(dead_code)] // This will likely be needed eventually.
    fn create_new_detached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        wrap_command(Command::new("tmux").args([
            "new-session",
            "-Ad",
            "-s",
            name,
            "-c",
            path.to_str().unwrap_or_default(),
        ]))
    }

    fn create_new_attached_attach_if_exists(name: &str, path: &Path) -> Result<Output> {
        wrap_command(Command::new("tmux").args([
            "new-session",
            "-A",
            "-s",
            name,
            "-c",
            path.to_str().unwrap_or_default(),
        ]))
    }

    fn create_new_detached(name: &str, path: &Path) -> Result<Output> {
        wrap_command(Command::new("tmux").args([
            "new-session",
            "-d",
            "-s",
            name,
            "-c",
            path.to_str().unwrap_or_default(),
        ]))
    }

    fn switch(to_name: &str) -> Result<Output> {
        wrap_command(Command::new("tmux").args(["switch-client", "-t", to_name]))
    }

    fn has_session(project_name: &str) -> bool {
        let output = wrap_command(Command::new("tmux").args([
            "has-session",
            "-t",
            &format!("={}", project_name),
        ]));

        output.is_ok_and(|o| o.status.success())
    }

    fn kill_session(project_name: &str) -> Result<()> {
        wrap_command(Command::new("tmux").args([
            "kill-session",
            "-t",
            &format!("={}", project_name),
        ]))?;
        Ok(())
    }

    fn in_session() -> bool {
        env::var("TMUX").is_ok()
    }
}
