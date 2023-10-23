[![mukduk:latest](https://github.com/pitoniak32/mukduk/actions/workflows/release.yml/badge.svg)](https://github.com/pitoniak32/mukduk/actions/workflows/release.yml)
## "R" is among the most menacing of sounds - Dwight K. Shrute.

Project management in your terminal should not be menacing.

pronounced - [muck duck]

## Dependencies
- [fzf](https://github.com/junegunn/fzf): used for the picker menus.
- [tmux](https://github.com/tmux/tmux)   (optional): required if you want to use tmux.
- [zellij](https://github.com/zellij-org/zellij) (optional): requried only if you want to use zellij.

## Usage

Open an fzf menu listing project dirs inside specified `projects_dir`, and then open multiplexer session with selected project.

You can set your projects_dir with two options:
- set `PROJECTS_DIR` env var.
- set `--projects-dir` flag.

```bash
mukduk project open -m tmux
```
```bash
mukduk --projects-dir="/absolute/path/to/proj_dir" project open -m tmux
```

OR you can specify a specific project to use.

```bash
mukduk project open -m tmux -p /absolute/path/to/project/dir -n name-other-than-dir-name
```
