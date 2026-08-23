use mlua::{Lua, Table};

const LUA_ITER: &str = include_str!("mod.lua");

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    let iter_tbl: mlua::Value = lua.load(LUA_ITER).eval()?;
    dale_tbl.set("iter", iter_tbl)
}
