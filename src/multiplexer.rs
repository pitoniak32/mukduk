use anyhow::Result;
use clap::ValueEnum;

use crate::{tmux::Tmux, zellij::Zellij, Project, ProjectArgs};

pub trait Multiplexer {
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
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
}
