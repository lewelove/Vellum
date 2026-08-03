//! Unit tests for Lua file system utilities (`vl.fs`).

use super::TempFile;
use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `vl.fs.exists` reports file existence correctly.
#[test]
fn test_vl_fs_exists() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("sample");
    let path_str = temp_file.path_str();

    let code = format!("return vl.fs.exists('{path_str}')");
    let exists: bool = engine.lua.load(&code).eval().expect("Execution failed");
    assert!(exists);

    let not_exists: bool = engine
        .lua
        .load("return vl.fs.exists('/tmp/non_existent_vellum_test_file.tmp')")
        .eval()
        .expect("Execution failed");
    assert!(!not_exists);
}

/// Verifies that `vl.fs.read` reads file contents and records file dependencies.
#[test]
fn test_vl_fs_read() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("test content");
    let path_str = temp_file.path_str();

    let code = format!("return vl.fs.read('{path_str}')");
    let content: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(content, "test content");

    let deps_code = format!("vl.fs.read('{path_str}')\nreturn __VELLUM_GET_DEPENDENCIES()");
    let deps: Vec<String> = engine.lua.load(&deps_code).eval().expect("Execution failed");
    assert!(deps.iter().any(|d| d.contains(&path_str)));

    let missing_code = "return vl.fs.read('/tmp/non_existent_vellum_test_file.tmp')";
    let missing_res: Option<String> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());
}

/// Verifies that `vl.fs.read_lines` reads non-comment lines and ignores missing paths.
#[test]
fn test_vl_fs_read_lines() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("first\n# comment\nsecond\n");
    let path_str = temp_file.path_str();

    let code = format!("return vl.fs.read_lines('{path_str}')");
    let lines: Vec<String> = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "first");
    assert_eq!(lines[1], "second");

    let missing_code = "return vl.fs.read_lines('/tmp/non_existent_vellum_test_file.tmp')";
    let missing_lines: Vec<String> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_lines.is_empty());
}

/// Verifies that `vl.fs.read_json` parses JSON files and reports invalid syntax.
#[test]
fn test_vl_fs_read_json() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let json_content = r#"{"artist":"Sample"}"#;
    let temp_file = TempFile::new(json_content);
    let path_str = temp_file.path_str();

    let code = format!("local t = vl.fs.read_json('{path_str}')\nreturn t.artist");
    let artist: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(artist, "Sample");

    let missing_code = "return vl.fs.read_json('/tmp/non_existent_vellum_test_file.tmp')";
    let missing_res: Option<Table> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let bad_file = TempFile::new("{invalid_json");
    let bad_path = bad_file.path_str();
    let bad_code = format!("return vl.fs.read_json('{bad_path}')");
    let bad_res = engine.lua.load(&bad_code).eval::<Table>();
    assert!(bad_res.is_err());
}

/// Verifies that `vl.fs.read_toml` parses TOML files and reports invalid syntax.
#[test]
fn test_vl_fs_read_toml() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let toml_content = r#"[album]
artist = "Sample""#;
    let temp_file = TempFile::new(toml_content);
    let path_str = temp_file.path_str();

    let code = format!("local t = vl.fs.read_toml('{path_str}')\nreturn t.album.artist");
    let artist: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(artist, "Sample");

    let missing_code = "return vl.fs.read_toml('/tmp/non_existent_vellum_test_file.tmp')";
    let missing_res: Option<Table> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let bad_file = TempFile::new("[album");
    let bad_path = bad_file.path_str();
    let bad_code = format!("return vl.fs.read_toml('{bad_path}')");
    let bad_res = engine.lua.load(&bad_code).eval::<Table>();
    assert!(bad_res.is_err());
}
