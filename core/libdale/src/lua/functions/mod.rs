pub mod r#fn;
pub mod fs;
pub mod get;
pub mod iter;
pub mod json;
pub mod list;
pub mod str;
pub mod system;
pub mod tbl;
pub mod toml;

use mlua::serde::SerializeOptions;
use mlua::{Lua, Table};

pub fn register_all(lua: &Lua, dale_tbl: &Table, opts: SerializeOptions) -> mlua::Result<()> {
    r#fn::register(lua, dale_tbl)?;
    fs::register(lua, dale_tbl, opts)?;
    get::register(lua, dale_tbl)?;
    iter::register(lua, dale_tbl)?;
    list::register(lua, dale_tbl)?;
    system::register(lua, dale_tbl)?;
    json::register(lua, dale_tbl, opts)?;
    str::register(lua, dale_tbl)?;
    tbl::register(lua, dale_tbl)?;
    toml::register(lua, dale_tbl, opts)?;
    Ok(())
}
