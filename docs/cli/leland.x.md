# leland x

## How it works

The `leland x` command is used to run **Leland Actions** for selected albums or the entire library. Each action is called by `leland x action_name`.

## Main Argument

#### `<name>`

Provides match to `[ll.actions].<name>` in `leland.lua`. If `<name>` matches:

- Loads `[ll.config].environment` file
- Reads the `<name>.run` path of the executable
- Executes it with JSON payload passed into its stdin

For cli ergonomics the `action_name` can be also called as `action-name` and it will still execute, because all of the `-` in name string will be replaced by `_` before it hitting the Lua key.

The action `<name>` must terminate the command, because it provides everything past it as options for the action itself. For example:

- `leland x action_name string --flag` -> will pass `"options": [ "string", "--flag" ]` in intermediary JSON
- `leland x --id "Album/Directory" action_name` -> will consume `--id` as `leland x` option

This way you can write actions that consume any kind of their own flags, options and arguments.

## Built-In Actions

Built-in actions are provided by leland binary out of the box.

- Configure them in `leland.lua` by `ll.actions({ <name> = { config = {} } })`.
- Override them entirely by `ll.actions({ <name> = { run = "" } })` key with your own scripts if you really want.

#### `intermediary`

Used to print JSON payload directly into stdout and exit. Useful for debug or development of new actions. Has no config.

#### `open-album-directory-in-terminal`

Opens the terminal emulator of choice, `cd`s into the album directory.

```lua
config = {
  -- sh -c "{open_with} --working-directory {album_directory}"
  -- defaults to native system terminal emulator
  open_with = "alacritty"
}
```

#### `open-album-directory-in-file-manager`

## Options

Options are used to specify which compiled albums (`album.lock.json` files) will be provided inside the JSON payload in form of `"albums": []` array. All options are mutually exclusive.

#### `--file` / `-f`

Provides an album by system path of either its `album.lock.json` or a directory containing this lock file. Default option, selected when no options given.

Argument: a path string. Defaults to `$(pwd)`.

Examples:

```bash
leland x -f '~/music/Path/To/Album/album.lock.json' action_name
```

```bash
leland x -f '~/music/Path/To/Album/' action_name
```

```bash
cd '~/music/Path/To/Album/'
leland x action_name
```

#### `--playing` / `-p`

Provides currently queued (playing or paused) album. Boolean, has no arguments.

Examples:

```bash
leland x -p action_name
```

```bash
leland x --playing action_name
```

#### `--id`

Provides an album by its `id` in the collection. The `id` is the album's directory path relative to library root. It is always present in compiled `album.lock.json`.

Argument: a string value of `[album.lock.json].album.id` key.

Examples:

```bash
leland x --id 'Library/Relative/Path/To/Album/Directory' action_name
```

#### `--query` / `-q`

Use this flag to select one or more albums from entire library using SQL that executes against in-memory SQLite database of the running server. Same `${}` shorthand expansion as in logic config can be used. The order passed from SQL is kept in `"albums": []` array.

Argument: an SQL string.

Examples:

```bash
# Selects albums with "total_tracks" > 20 and orders them by "date_added"
leland x -q '
WHERE ${album.total_tracks} > 20
ORDER BY ${album.date_added}
' action_name
```

```bash
# Selects albums with "Brian Eno" in "albumartist" key and orders them by "original_date"
leland x --query '
${album.albumartist} LIKE %{Brian Eno}
ORDER BY ${album.info.original_date} DESC
' action_name
```

## Specifications

### Intermediary JSON

The payload JSON contains three fields: `"albums"`, `"config"` and `"options"`

```jsonc
{
  // array populated with album.lock.json files
  "albums": [
    // 1st album.lock.json
    {
      "album": {
        // ...
      },
      "tracks": [
        // ...
      ]
    },
    // 2nd album.lock.json
    {
      // etc...
    }
    // etc...
  ],

  "config": {
    // `ll.config({})` table -> json
    "leland": {},
    // `ll.actions({ <name> = { config = {} } })` table -> json
    "action": {},
  },

  // string passed after `action_name` -> separated by space -> array
  "options": []
}
```

### Lua Config

For action to become executable it must be registered in `leland.lua` config. 

```lua
ll.actions({

  action_name = {

    -- path to override expected executable action binary
    -- defaults to `~/.local/share/leland/actions/{name}/{name}
    run = "",

    -- the config table
    -- can be populated with any static data
    -- converted to JSON and sent as a part of stdin payload
    config = {}, 
  }
})
```

### API

Each action can be executed from any interface by `/api/actions/{name}?{option}={argument}` endpoint.
