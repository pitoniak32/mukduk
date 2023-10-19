use std::process::{Command, Output, Stdio};

use anyhow::Result;

pub fn wrap_command(command: &mut Command) -> Result<Output> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if output.status.success() {
        log::info!("{}", String::from_utf8_lossy(&output.stdout).trim());
    } else {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(output)
}
