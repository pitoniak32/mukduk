use anyhow::Result;
use cli::MukdukCli;

mod config;
mod helper;
mod multiplexer;
mod project;

mod fzf;

mod cli;
mod commands;

fn main() -> Result<()> {
    let cli = MukdukCli::init()?;

    cli.handle_cmd()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::project::Project;

    #[test]
    fn should_update_project_name_with_underscores() {
        assert_eq!(
            Project::new(PathBuf::from(""), ".test.test".to_string()).get_name(),
            "_test_test".to_string()
        )
    }
}
