use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    path::Path,
    process::{Command, Output},
};

use crate::{helper::wrap_command, Project, ProjectArgs};

pub struct Zellij;

impl Zellij {
    pub fn open(proj_args: &ProjectArgs, project: Project) -> Result<()> {
        log::info!(
            "creating {:?} session with project: {:?}!",
            proj_args.multiplexer,
            project
        );

        if Zellij::not_in() {
            Zellij::create_attached(&project.name, &project.path)?;
        } else {
            eprintln!("{}", "\nZellij does not currently have support for switching sessions while inside an active session.\n\nTry detaching from your current session, and try again.\n".yellow().bold())
        }

        Ok(())
    }
}

impl Zellij {
    fn create_attached(name: &str, path: &Path) -> Result<Output> {
        wrap_command(
            Command::new("zellij")
                .args(["a", "-c", name])
                .current_dir(path.to_str().unwrap_or_default()),
        )
    }

    #[allow(dead_code)] // This will likely be needed eventually.
    fn has_session(project_name: &str) -> bool {
        let output = Command::new("zellij")
            .arg("ls")
            .output()
            .expect("zellij was not able to print list of sessions.");
        match output.status.success() {
            true => {
                assert_ne!(project_name, "", "Zellij session name cannot be empty. The sessions list will contain \"\" due to split('\n').");
                if String::from_utf8_lossy(&output.stdout)
                    .split('\n')
                    .any(|session_name| session_name == project_name)
                {
                    return true;
                }
                false
            }
            false => {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                if error_msg.contains("No active zellij sessions found.") {
                    false
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
