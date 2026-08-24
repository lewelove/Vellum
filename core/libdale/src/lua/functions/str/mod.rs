#[cfg(test)]
mod tests;

use mlua::{Lua, Table};

const LUA_STR: &str = include_str!("mod.lua");

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    let str_table: Table = lua.load(LUA_STR).eval()?;
    dale_tbl.set("str", str_table)
}
