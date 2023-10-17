Will be used for managing local terminal development environment.

To list project directories under `PROJ_DIR` env var, and then open a multiplexer session with selected project.
```bash
mk project open -m tmux
```
OR you can specify a specific project to use.
```
mk project open -m tmux -p /absolute/path/to/project/dir -n <name of project session (default project dir name)>
```