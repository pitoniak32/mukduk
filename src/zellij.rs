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
    fn has_session(project_name: &str) -> bool {
        todo!();
        // match Command::new("zellij")
        //     .args([""])
        //     .status()
        // {
        //     Ok(status) => {
        //         if status.success() {
        //             true
        //         } else {
        //             false
        //         }
        //     }
        //     Err(_) => false,
        // }
    }

    fn not_in() -> bool {
        match env::var("ZELLIJ") {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}
