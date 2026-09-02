# Dale Milestones

This document describes the system design and architecture milestones I have in mind for Dale.

## Lua Driven Manifest Engine

The `dale manifest` should be a Lua function. It is designed to manifest data from physical filesystem state into plaintext files. The definition is:

```lua
d.manifest("name", {
  cluster = function(dir) return { dir } end -- default implementation if omitted
  generate = function(dir) 
    return {
      file = "manifest_name.toml/json",
      content = "" or {} -- either string to validate and write verbatim or table to be serialized
    }
  end
})
```

The `cluster` runs first. It takes the directory path from `--dir` option in CLI call. It must return an array of directory paths. All paths are canonicalized by Rust and validated to be directories and exist.

Then for every directory path in this array runs `generate`. It always returns Lua table containing two keys: `file` and `content`.

```lua
```
