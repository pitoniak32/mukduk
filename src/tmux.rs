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
    pub fn open(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "attempting to open {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );
        if Tmux::not_in() {
            Tmux::create_new_attached_attach_if_exists(&project.name, &project.path)?;
        } else if Tmux::has_session(&project.name) {
            log::info!("Session '{}' already exists, opening.", project.name);
            Tmux::switch(&project.name)?;
        } else {
            log::info!(
                "Session '{}' does not already exist, creating and opening.",
                project.name
            );

            if Tmux::create_new_detached(&project.name, &project.path)
                .is_ok_and(|o| o.status.success())
            {
                Tmux::switch(&project.name)?;
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
                log::info!("killed {}", s)
            } else {
                log::error!("error while killing {}", s)
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

    fn not_in() -> bool {
        env::var("TMUX").is_err()
    }
}
