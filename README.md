## "R" is among the most menacing of sounds - Dwight K. Shrute.

Project management in your terminal should not be menacing.

pronounced - [muck duck]

---

To list project directories under `PROJ_DIR` env var, and have the user select the project. Then open a multiplexer session with selected project.
```bash
mukduk project open -m tmux
```

OR you can specify the project directory by `-p` flag rather than env var. Which will let you manually select still.

```bash
mukduk -p /absolute/path/to/proj_dir project open -m tmux
```

OR you can specify a specific project to use.

```bash
mukduk project open -m tmux -p /absolute/path/to/project/dir -n name-other-than-dir-name
```