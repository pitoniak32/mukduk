use anyhow::Result;
use colored::Colorize;
use std::{env, process::Command};

use crate::{Project, ProjectArgs};

pub struct Zellij;

impl Zellij {
    pub fn create(proj_args: &ProjectArgs, project: Project) -> anyhow::Result<()> {
        log::info!(
            "creating {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );

        if Zellij::not_in() {
            // Will attach to an existing session, or create a new one if one does not exist.
            Command::new("zellij")
                .args(["attach", "-c", &project.name])
                .current_dir(project.path.to_str().unwrap_or_default())
                .status()?;
        } else {
            eprintln!("{}", "\nZellij does not currently have support for switching sessions while inside an active session.\n\nTry detaching from your current session, and try again.\n".yellow().bold())
        }

        Ok(())
    }

    pub fn open(proj_args: &ProjectArgs, project: Project) -> anyhow::Result<()> {
        log::info!(
            "opening {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );
        // Will attach to an existing session, or create a new one if one does not exist.
        Command::new("zellij")
            .args(["attach", "-c", &project.name])
            .current_dir(project.path.to_str().unwrap_or_default())
            .status()?;

        Ok(())
    }
}

impl Zellij {
    #[allow(dead_code)]
    fn has_session(project_name: &str) -> Result<bool> {
        let output = Command::new("zellij").arg("ls").output()?;
        match output.status.success() {
            true => {
                if String::from_utf8_lossy(&output.stdout)
                    .split('\n')
                    .any(|session_name| session_name == project_name)
                {
                    return Ok(true);
                }
                Ok(false)
            }
            false => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                if error_msg.contains("No active zellij sessions found.") {
                    Ok(false)
                } else {
                    eprintln!(
                        "zellij command failed with exit code: {}, and error msg: {}\n",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    panic!("The command 'zellij ls' should not fail unless zellij is not present on the machine.")
                }
            }
        }
    }

    fn not_in() -> bool {
        env::var("ZELLIJ").is_err()
    }
}
