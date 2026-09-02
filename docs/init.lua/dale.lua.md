# dale.lua

This file describes and outlines future documentation of dale configuration written in lua.

---

The `~/.config/dale/dale.lua` is the initial file that must exist for config to be active.

## require()

For modularizing config use `require("name")` of the `name.lua` files reative to `~/.config/dale/`. For path specification use `.` as the delimiter.

```lua

-- imports ~/.config/dale/module.lua
require("module")

-- imports ~/.config/dale/folder/module.lua
require("folder.module")
```
