# ll.interfaces

This function provides the data for `leland interface` to run.

## Specifications

### Execution:

Each interface is ran by `leland interface interface_name`. For cli ergonomics the `interface_name` can be also called as `interface-name` and it will still run, because all of the `-` in name string will be replaced by `_` before it hitting the lua key.

### Lua specification:

```lua
ll.interfaces({

  -- the `default` interface name
  -- run with `leland interface` or `leland interface default`
  -- always exists, can be overwritten
  default = {

    -- path to override expected executable interface binary
    -- defaults to `~/.local/share/leland/interfaces/{name}/run.sh`
    run = "",

    -- the config table
    -- can be populated with any static data
    -- converted to JSON and sent as `/api/interfaces/{name}/config` endpoint
    config = {}, 

    -- assets table
    -- can be populated with system paths
    -- used to serve files to ui via /api/interfaces/{name}/assets/ endpoint
    assets = {
        -- if path == file
        --   serve the file with MIME type resolved directly by {file_name}
        --   /api/interfaces/{name}/assets/{file_name}
        file_name = "/path/to/file.ext"

        -- if path == directory
        --   serve the file with MIME type resolved from directory path by {dir_name} + {subpath from api call}
        --   /api/interfaces/{name}/assets/{dir_name}/{file path relative to dir_name}
        dir_name = "/path/to/directory/"
    },
  }
})
```

## Examples

The `{name} = true` can be used to enable interface with default run path and no config:

```lua
ll.interfaces({
  -- inherits `run = "~/.local/share/leland/interfaces/interface_name/run.sh"`
  -- no config api
  interface_name = true
})
```

Also the `config = {}` and `run = "path/to/run"` can be individually provided to enable it:

```lua
ll.interfaces({
  -- inherits `run = "~/.local/share/leland/interfaces/interface_name/run.sh"`
  -- `/api/interfaces/interface_name/config` will return `{ "theme": "dark" }` JSON
  interface_name = { config = { theme = "dark" } }
})
```

```lua
local repo_path = "~/dev/leland/"

ll.interfaces({
  -- no config api
  interface_name = { run = repo_path .. "interfaces/example/cool_ui.sh" }
})
```
