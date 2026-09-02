# codebase

- Remove everything hardcoded from Rust
- Rewrite ALL default key resolution logic from Rust to Lua

# engine

- Make compilation failure propagate the compile error in server output, on compilation triggered by ANY cause
- Rename `metadata.toml` -> `album.toml`
- Make `embedded` array a standard builtin part of `tracks`
- Make error messages beautiful

# cli

### manifest

- Make `dale manifest` a command you execute for each individual album specifically

### compile

- Make `compile` bypass updating and print result straight in `stdin`
- Add `--directory / -d` flag to spec the target album
- Make it impossible to compile > 1 album, print invalid directory error when 
- Add subcommand (`dale compile album` / `dale compile library`) that will compile either single album in pwd or all albums across library root
- Add `--force` flag

### update

- Reject target album if it's not in `storage.library`

# theming

- Make absolutely all white elements derived from single `oklab` white value using alpha channel
- Add new object that tracks currently playing OR last played album at all times to always have something to display in `queueview`
- Remove the inactive elements from queue sidebar

# api

- Add `/mpd/` prefix to all mpd control related api endpoint. Example: `/api/mpd/play_album/`

# actions

- Remove any kind of terminal messages printing unless `--debug` is used
- Make `dale x` drop into process output just like the `dale interface` does

### cover-palette

- Add little run.sh script that executes actual binary and then opens the file generated

### open-album-directory-in-terminal

- Built-in action
- Trigger on `Ctrl + T` in album drawer

### open-album-directory-in-file-manager

- Built-in action
- Trigger on `Ctrl + D` in album drawer

# config

