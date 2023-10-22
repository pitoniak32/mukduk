use anyhow::Result;
use clap::ValueEnum;

use crate::{tmux::Tmux, zellij::Zellij, Project, ProjectArgs};

pub trait Multiplexer {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
    fn get_sessions(self) -> Vec<String>;
    fn kill_sessions(self, sessions: Vec<String>) -> Result<()>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Multiplexers {
    Tmux,
    Zellij,
}

impl Multiplexer for Multiplexers {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()> {
        match self {
            Multiplexers::Tmux => {
                Tmux::open(proj_args, project)?;
            }
            Multiplexers::Zellij => {
                Zellij::open(proj_args, project)?;
            }
        }
        Ok(())
    }

    fn get_sessions(self) -> Vec<String> {
        match self {
            Multiplexers::Tmux => Tmux::list_sessions(),
            Multiplexers::Zellij => Zellij::list_sessions(),
        }
    }

    fn kill_sessions(self, sessions: Vec<String>) -> Result<()> {
        match self {
            Multiplexers::Tmux => Tmux::kill_sessions(&sessions),
            Multiplexers::Zellij => Zellij::kill_sessions(&sessions),
        }
    }
}
