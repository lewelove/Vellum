#[cfg(test)]
mod tests;

use mlua::serde::SerializeOptions;
use mlua::{Lua, LuaSerdeExt, Table};

pub fn register(lua: &Lua, dale_tbl: &Table, opts: SerializeOptions) -> mlua::Result<()> {
    let json_table = lua.create_table()?;
    json_table.set(
        "decode",
        lua.create_function(move |lua, s: String| {
            let val: serde_json::Value = serde_json::from_str(&s)
                .map_err(mlua::Error::external)?;
            lua.to_value_with(&val, opts)
        })?,
    )?;

    json_table.set(
        "encode",
        lua.create_function(|lua, val: mlua::Value| {
            let json_val: serde_json::Value = lua.from_value(val)?;
            serde_json::to_string(&json_val).map_err(mlua::Error::external)
        })?,
    )?;

    dale_tbl.set("json", json_table)
}
