use anyhow::Result;
use clap::ValueEnum;

use crate::{tmux::Tmux, zellij::Zellij, Project, ProjectArgs};

pub trait Multiplexer {
    fn create(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
    fn open(self, proj_args: &ProjectArgs, project: Project) -> Result<()>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Multiplexers {
    Tmux,
    Zellij,
}

impl Multiplexer for Multiplexers {
    fn create(self, proj_args: &ProjectArgs, project: Project) -> Result<()> {
        match self {
            Multiplexers::Tmux => {
                Tmux::create(proj_args, project)?;
            }
            Multiplexers::Zellij => {
                Zellij::create(proj_args, project)?;
            }
        }
        Ok(())
    }

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
