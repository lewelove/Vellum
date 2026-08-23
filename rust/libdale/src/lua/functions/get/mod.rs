#[cfg(test)]
mod tests;

use mlua::{Lua, Table};

const LUA_GET: &str = include_str!("mod.lua");

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    let get_fn: mlua::Function = lua.load(LUA_GET).eval()?;
    dale_tbl.set("get", get_fn)
}
