use anyhow::Result;
use std::{
    env,
    process::{Command, ExitStatus, Stdio},
};

use crate::{Project, ProjectArgs};

pub struct Tmux;

impl Tmux {
    pub fn create(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "creating {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );

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
            log::info!(
                "Session '{}' has been created in '{}'.",
                project.name,
                project.path.to_string_lossy()
            );
        } else {
            eprintln!("Session failed to be created with exit_code: {}", output);
        }

        Ok(())
    }

    pub fn open(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "opening {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );
        let output: ExitStatus;
        if Tmux::not_in() {
            let _output = Command::new("tmux")
                .args([
                    "new-session",
                    "-A",
                    "-s",
                    &project.name,
                    "-c",
                    project.path.to_str().unwrap_or_default(),
                ])
                .status()?;
        } else if Tmux::has_session(&project.name) {
            log::info!("Session '{}' already exists, opening.", project.name);
            let _child = Command::new("tmux")
                .args(["switch-client", "-t", &project.name])
                .status()?;
        } else {
            log::info!(
                "Session '{}' does not already exist, creating and opening.",
                project.name
            );

            let output_tmux = Command::new("tmux")
                .stdout(Stdio::piped())
                .args([
                    "new-session",
                    "-d",
                    "-s",
                    &project.name,
                    "-c",
                    project.path.to_str().unwrap_or_default(),
                ])
                .spawn()?.wait_with_output()?;

            if output_tmux.status.success() {
                log::info!("{}", String::from_utf8_lossy(&output_tmux.stdout));
                let output = Command::new("tmux")
                    .args(["switch-client", "-t", &project.name])
                    .stdout(Stdio::piped())
                    .spawn()?.wait_with_output()?;
            } else {
                eprintln!("Session failed to be opened with exit_code: {:?}", output_tmux.status.code());
            }
        }

        Ok(())
    }
}

impl Tmux {
    fn has_session(project_name: &str) -> bool {
        log::info!("has-session");
        match Command::new("tmux")
            .args(["has-session", "-t", &format!("={}", project_name)])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(output) => {
                match output.wait_with_output() {
                    Ok(wout) => {
                        if wout.status.success() {
                            log::info!("{}", String::from_utf8_lossy(&wout.stdout).trim());
                            true
                        } else {
                            log::warn!("{}", String::from_utf8_lossy(&wout.stderr).trim());
                            false
                        }
                    },
                    Err(_) => {
                        false
                    },
                }
            }, 
            Err(_) => false,
        }
    }

    fn not_in() -> bool {
        env::var("TMUX").is_err()
    }
}
