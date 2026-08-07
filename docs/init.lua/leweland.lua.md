# leland.lua

This file describes and outlines future documentation of leland configuration written in lua.

---

The `~/.config/leland/leland.lua` is the initial file that must exist for config to be active.

## require()

For modularizing config use `require("name")` of the `name.lua` files reative to `~/.config/leland/`. For path specification use `.` as the delimiter.

```lua

-- imports ~/.config/leland/module.lua
require("module")

-- imports ~/.config/leland/folder/module.lua
require("folder.module")
```
