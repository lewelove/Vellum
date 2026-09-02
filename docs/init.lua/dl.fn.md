# dl.fn

This document describes a built-in set of `dl.fn` functions that can be used *inside* `function()` provided by any `dl.compile` to execute useful logic without lots of Lua boilerplate.

**Specifications:**
- For all `fn` consuming a system path string Rust canonicalizes the system path. If string is passed as relative path it joins it to `__DALE_ACTIVE_ROOT` variable (the root of album being compiled).

### dl.fn.type_check

Checks input value (usually provided by `manifests` table) for one of Dale Types, else throws error. Can be used on empty (nil or "") values, which will pass it, as this logic is delegated to `dl.fn.require()`.

```lua
dl.fn.type_check(value, "dale_type")
```

### dl.fn.require

Takes Lua value and checks if the value passed is `nil` or `""`. If false returns value. If true throws compilation error.

```lua
dl.fn.require(value)
```

### dl.fn.hash_string

Takes a string as input and returns a BLAKE3 hash of string itself. Useful for static hash generation from album data, for specific sort for example.

```lua
dl.compile.album.key({ cool_id = function(ctx, m)
  local key = m.metadata.album.album .. m.metadata.album.albumartist .. m.metadata.album.date
  return dl.fn.hash_string(key)
end })
```

### dl.fn.hash_file

Takes a system path string as input and returns a BLAKE3 hash of the file. Cannot be a directory.

```lua
dl.compile.album.key({ cool_id = function(ctx, m)
  local key = m.metadata.album.album .. m.metadata.album.albumartist .. m.metadata.album.date
  return dl.fn.hash_string(key)
end })
```

### dl.fn.to_table

Takes system path string of a JSON/TOML file as input and returns Lua table. Useful for pulling non-manifest data files to the compilation context.

**Specifications:**
- Reads the `.ext` of the file, matches the JSON/TOML based on it. Validates the file.
- Converts to Lua table, returns it.

### dl.fn.read_text

Takes the system path string of any file and returns a literal string. Useful for pulling literally any text data to the compilation context, like lyrics or notes.
