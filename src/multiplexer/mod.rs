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

    fn unique_session(self) -> Result<()> {
        match self {
            Multiplexers::Tmux => Tmux::unique_session(),
            Multiplexers::Zellij => todo!(),
        }
    }
}
