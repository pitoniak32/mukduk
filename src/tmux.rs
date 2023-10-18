use anyhow::Result;
use std::{
    env,
    process::{Command, ExitStatus},
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
                    "-Ad",
                    "-s",
                    &project.name,
                    "-c",
                    project.path.to_str().unwrap_or_default(),
                ])
                .status()?;
        } else {
            if Tmux::has_session(&project.name) {
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

        Ok(())
    }
}

impl Tmux {
    fn has_session(project_name: &str) -> bool {
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

    fn not_in() -> bool {
        match env::var("TMUX") {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
