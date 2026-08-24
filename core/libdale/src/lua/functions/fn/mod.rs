#[cfg(test)]
mod tests;

use mlua::{Lua, Table};

const LUA_FN: &str = include_str!("mod.lua");

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    let fn_tbl: Table = lua.load(LUA_FN).eval()?;
    dale_tbl.set("fn", fn_tbl)
}
