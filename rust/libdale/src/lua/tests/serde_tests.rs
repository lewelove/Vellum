//! Unit tests for JSON and TOML serialization utilities (`d.json` and `d.toml`).

use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `d.json.decode` and `d.json.encode` convert data structures correctly.
#[test]
fn test_dl_json_encode_decode() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let json_input = r#"{"title":"Album","year":2024}"#;

    let code = format!("return d.json.decode([==[{json_input}]==])");
    let decoded: Table = engine.lua.load(&code).eval().expect("Execution failed");
    let title: String = decoded.get("title").expect("Missing title");
    let year: u32 = decoded.get("year").expect("Missing year");
    assert_eq!(title, "Album");
    assert_eq!(year, 2024);

    let encoded: String = engine
        .lua
        .load("return d.json.encode({ title = 'Album', year = 2024 })")
        .eval()
        .expect("Execution failed");
    assert!(encoded.contains("\"title\":\"Album\""));
    assert!(encoded.contains("\"year\":2024"));
}

/// Verifies that `d.json.decode` fails on malformed JSON text.
#[test]
fn test_dl_json_decode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.json.decode('{bad_json:')").eval::<Table>();
    assert!(result.is_err());
}

/// Verifies that `d.json.encode` fails when given non-serializable Lua values.
#[test]
fn test_dl_json_encode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.json.encode({ func = function() end })").eval::<String>();
    assert!(result.is_err());
}

/// Verifies that `d.toml.decode` and `d.toml.encode` convert data structures correctly.
#[test]
fn test_dl_toml_encode_decode() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let toml_input = r#"[album]
title = "Demo""#;

    let code = format!("return d.toml.decode([==[{toml_input}]==])");
    let decoded: Table = engine.lua.load(&code).eval().expect("Execution failed");
    let album: Table = decoded.get("album").expect("Missing album section");
    let title: String = album.get("title").expect("Missing title");
    assert_eq!(title, "Demo");

    let encoded: String = engine
        .lua
        .load("return d.toml.encode({ album = { title = 'Demo' } })")
        .eval()
        .expect("Execution failed");
    assert!(encoded.contains("[album]"));
    assert!(encoded.contains("title = \"Demo\""));
}

/// Verifies that `d.toml.decode` fails on malformed TOML text.
#[test]
fn test_dl_toml_decode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.toml.decode('[album')").eval::<Table>();
    assert!(result.is_err());
}

/// Verifies that `d.toml.encode` fails when given non-serializable Lua values.
#[test]
fn test_dl_toml_encode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.toml.encode({ func = function() end })").eval::<String>();
    assert!(result.is_err());
}
