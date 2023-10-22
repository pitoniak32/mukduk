use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use anyhow::Result;
use colored::Colorize;

use crate::{config::ConfigEnvKey, Project};

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
    projects_dir: Option<PathBuf>,
    project_dir: &Option<PathBuf>,
    name: Option<String>,
) -> Result<Project> {
    let project: Project;
    if let Some(selected_project) = project_dir {
        project = Project {
            name: name.unwrap_or(
                selected_project
                    .file_name()
                    .expect("file_name should be representable as a String.")
                    .to_string_lossy()
                    .to_string(),
            ),
            path: selected_project.clone(),
        }
    } else {
        project = pick_project(projects_dir)?;
    }

    Ok(project)
}

pub fn pick_project(projects_dir: Option<PathBuf>) -> Result<Project> {
    let proj_dir: PathBuf = projects_dir.unwrap_or(PathBuf::from(ConfigEnvKey::ProjDir));

    log::info!("Using project_dir: {:?}", &proj_dir);

    let projects: Vec<_> = get_directories(proj_dir)?
        .iter()
        .map(|d| Project {
            path: d.to_path_buf(),
            name: d
                .file_name()
                .expect("file_name should be representable as a String")
                .to_string_lossy()
                .to_string(),
        })
        .collect();
    let project_name = fzf_get_project_name(
        &projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;

    if project_name.is_empty() {
        eprintln!("\n{}\n", "No project was selected.".red().bold());
        std::process::exit(1);
    }
    Ok(projects.iter().find(|p| p.name == project_name).expect("This should never be None since the project_names list only contains names from the list of projects. If the user does not choose from the list the program will exit.").clone())
}

pub fn fzf_get_project_name(project_names: &str) -> Result<String> {
    let echo_child = Command::new("echo")
        .arg(project_names)
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(echo_stdout) = echo_child.stdout {
        let fzf_child = Command::new("fzf")
            .stdin(echo_stdout)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?;
        let selected_name = String::from_utf8_lossy(&fzf_child.stdout)
            .trim()
            .to_string();
        return Ok(selected_name);
    }
    Ok("".to_string())
}

pub fn fzf_get_sessions(session_names: Vec<String>) -> Result<Vec<String>> {
    if session_names.is_empty() {
        eprintln!("\n{}\n", "No sessions found to choose from.".blue().bold());
        std::process::exit(0);
    }
    let echo_child = Command::new("echo")
        .arg(session_names.join("\n"))
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(echo_stdout) = echo_child.stdout {
        let fzf_child = Command::new("fzf")
            .args(["--phony", "--multi"])
            .stdin(echo_stdout)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        if fzf_child.status.success() {
            let selected_names: Vec<_> = String::from_utf8_lossy(&fzf_child.stdout)
                .trim_end()
                .split('\n')
                .map(|s| s.to_string())
                .collect();
            return Ok(selected_names);
        } else {
            return Ok(vec![]);
        }
    }
    Ok(vec![])
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
