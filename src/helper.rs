use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use anyhow::Result;
use colored::Colorize;

use crate::{fzf::FzfCmd, Project};

pub fn wrap_command(command: &mut Command) -> Result<Output> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    // Use log crate to allow verbosity flag to control wrapped command logs.
    if output.status.success() && !output.stdout.is_empty() {
        log::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else if !output.stderr.is_empty() {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output)
}

pub fn get_project(
    projects_dir: PathBuf,
    project_dir: &Option<PathBuf>,
    name: Option<String>,
) -> Result<Project> {
    if let Some(selected_project) = project_dir {
        Ok(Project::new(
            selected_project.clone(),
            name.unwrap_or(
                selected_project
                    .file_name()
                    .expect("selected project should have a valid file / dir name.")
                    .to_string_lossy()
                    .to_string(),
            ),
        ))
    } else {
        pick_project(projects_dir)
    }
}

pub fn pick_project(proj_dir: PathBuf) -> Result<Project> {
    log::info!("Using project_dir: {:?}", &proj_dir);

    let projects: Vec<_> = get_directories(proj_dir)?
        .iter()
        .map(|d| {
            Project::new(
                d.to_path_buf(),
                d.file_name()
                    .expect("file_name should be representable as a String")
                    .to_string_lossy()
                    .to_string(),
            )
        })
        .collect();

    let project_names = projects.iter().map(|p| p.name.clone()).collect::<Vec<_>>();

    log::debug!("projects: {projects:#?}");

    let project_name = FzfCmd::new().find_vec(project_names)?;

    if let Some(project) = projects.iter().find(|p| p.name == project_name) {
        Ok(project.clone())
    } else {
        eprintln!("{}", "No project was selected.".red().bold());
        std::process::exit(1);
    }
}

pub fn fzf_get_sessions(session_names: Vec<String>) -> Result<Vec<String>> {
    if session_names.is_empty() {
        eprintln!("\n{}\n", "No sessions found to choose from.".blue().bold());
        std::process::exit(0);
    }

    Ok(FzfCmd::new()
        .args(vec!["--phony", "--multi"])
        .find_vec(session_names)?
        .trim_end()
        .split('\n')
        .map(|s| s.to_string())
        .collect())
}

pub fn get_directories(path: PathBuf) -> Result<Vec<PathBuf>> {
    Ok(fs::read_dir(path)?
        .filter_map(|dir| match dir {
            Ok(dir) => match dir.file_type() {
                Ok(ft) => {
                    if ft.is_dir() {
                        Some(dir.path())
                    } else {
                        None
                    }
                }
                Err(err) => {
                    eprintln!("An error occurred, skipping entry: {err}");
                    None
                }
            },
            Err(err) => {
                eprintln!("An error occurred, skipping entry: {err}");
                None
            }
        })
        .collect())
}
