use anyhow::Result;
use clap::ValueEnum;

use crate::{commands::project::ProjectArgs, Project};

use self::{tmux::Tmux, zellij::Zellij};

pub mod tmux;
pub mod zellij;

pub trait Multiplexer {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
    fn get_sessions(self) -> Vec<String>;
    fn kill_sessions(self, sessions: Vec<String>) -> Result<()>;
    fn unique_session(self) -> Result<()>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Multiplexers {
    Tmux,
    Zellij,
}

impl Multiplexer for Multiplexers {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()> {
        match self {
            Self::Tmux => {
                Tmux::open(proj_args, project)?;
            }
            Self::Zellij => {
                Zellij::open(proj_args, project)?;
            }
        }
        Ok(())
    }

    fn get_sessions(self) -> Vec<String> {
        match self {
            Self::Tmux => Tmux::list_sessions(),
            Self::Zellij => Zellij::list_sessions(),
        }
    }

    fn kill_sessions(self, sessions: Vec<String>) -> Result<()> {
        match self {
            Self::Tmux => Tmux::kill_sessions(&sessions),
            Self::Zellij => Zellij::kill_sessions(&sessions),
        }
    }

    fn unique_session(self) -> Result<()> {
        match self {
            Self::Tmux => Tmux::unique_session(),
            Self::Zellij => todo!(),
        }
    }
}
