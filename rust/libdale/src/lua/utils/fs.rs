use crate::error::DaleError;
use crate::lua::EngineContext;
use mlua::serde::SerializeOptions;
use mlua::{Lua, LuaSerdeExt, Table};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn parse_path_with_ctx(ctx: &EngineContext, path_str: Option<String>) -> Option<PathBuf> {
    let raw_path = path_str?;
    if raw_path.trim().is_empty() {
        return None;
    }
    let path = crate::utils::expand_path(&raw_path);
    ctx.record_dependency(&path);
    Some(path)
}

fn parse_and_record_path(lua: &Lua, path_str: Option<String>) -> Option<PathBuf> {
    let ctx = lua.app_data_ref::<EngineContext>()?;
    parse_path_with_ctx(&ctx, path_str)
}

fn fs_exists(lua: &Lua, path_str: Option<String>) -> bool {
    let Some(path) = parse_and_record_path(lua, path_str) else {
        return false;
    };
    path.exists()
}

fn fs_read(lua: &Lua, path_str: Option<String>) -> mlua::Result<Option<String>> {
    let Some(path) = parse_and_record_path(lua, path_str) else {
        return Ok(None);
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(mlua::Error::external(err)),
    }
}

fn fs_read_lines(lua: &Lua, path_str: Option<String>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let Some(path) = parse_and_record_path(lua, path_str) else {
        return Ok(table);
    };

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(table),
        Err(err) => return Err(mlua::Error::external(err)),
    };

    let reader = BufReader::new(file);
    let mut idx = 1;
    for line_res in reader.lines() {
        let line = line_res.map_err(mlua::Error::external)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            table.set(idx, trimmed)?;
            table.set(trimmed, idx)?;
            idx += 1;
        }
    }

    Ok(table)
}

fn create_fs_reader(
    lua: &Lua,
    opts: SerializeOptions,
    expected_ext: &'static str,
) -> mlua::Result<mlua::Function> {
    lua.create_function(move |lua, path_str: Option<String>| {
        let Some(ctx) = lua.app_data_ref::<EngineContext>() else {
            return Err(mlua::Error::external(anyhow::anyhow!(
                "EngineContext is not initialized"
            )));
        };
        let Some(path) = parse_path_with_ctx(&ctx, path_str) else {
            return Ok(mlua::Value::Nil);
        };
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !ext.eq_ignore_ascii_case(expected_ext) {
            return Err(mlua::Error::external(DaleError::InvalidFileExtension {
                path,
                expected: expected_ext.to_string(),
            }));
        }
        match crate::cache::read_object_cached(&path, &ctx.cache_root) {
            Ok(json_val) => lua.to_value_with(&json_val, opts),
            Err(DaleError::ManifestIoError(err))
                if err.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(mlua::Value::Nil)
            }
            Err(e) => Err(mlua::Error::external(e)),
        }
    })
}

pub fn create_fs_table(lua: &Lua, opts: SerializeOptions) -> mlua::Result<Table> {
    let fs_table = lua.create_table()?;
    fs_table.set(
        "exists",
        lua.create_function(|lua, path_str: Option<String>| Ok(fs_exists(lua, path_str)))?,
    )?;
    fs_table.set(
        "read",
        lua.create_function(|lua, path_str: Option<String>| fs_read(lua, path_str))?,
    )?;
    fs_table.set(
        "read_lines",
        lua.create_function(|lua, path_str: Option<String>| fs_read_lines(lua, path_str))?,
    )?;
    fs_table.set(
        "read_json",
        create_fs_reader(lua, opts, "json")?,
    )?;
    fs_table.set(
        "read_toml",
        create_fs_reader(lua, opts, "toml")?,
    )?;
    Ok(fs_table)
}
