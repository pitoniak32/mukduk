use anyhow::Result;
use std::{
    ffi::OsStr,
    fmt::Display,
    io::Write,
    process::{Command, Stdio},
};

pub struct FzfCmd {
    command: Command,
}

impl FzfCmd {
    pub fn new() -> Self {
        Self {
            command: Command::new("fzf"),
        }
    }

    pub fn _arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn find_vec<T>(&mut self, input: Vec<T>) -> Result<String>
    where
        T: AsRef<OsStr> + Display,
    {
        let projects_string: String = input.iter().fold(String::new(), |acc, project_name| {
            format!("{acc}\n{project_name}")
        });
        self.find_string(projects_string.trim_start())
    }

    pub fn find_string(&mut self, input: &str) -> Result<String> {
        let mut fzf_child = self
            .command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("fzf command should spawn");

        // Get the stdin handle of the child process
        if let Some(stdin) = &mut fzf_child.stdin {
            // Write your input string to the command's stdin
            stdin
                .write_all(input.as_bytes())
                .expect("should be able to pass project names to fzf stdin");
        } else {
            eprintln!("Failed to get stdin handle for the child process");
        }

        // Ensure the child process has finished
        let output = fzf_child.wait_with_output()?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        Ok("".to_string())
    }
}
