use mlua::{Lua, Table};

const LUA_LIST: &str = include_str!("mod.lua");

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    let list_tbl: Table = lua.load(LUA_LIST).eval()?;
    dale_tbl.set("list", list_tbl)
}
