//! Unit tests for Lua file system utilities (`d.fs`).

use super::TempFile;
use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `d.fs.exists` reports file existence correctly and returns `false` on missing or nil paths.
#[test]
fn test_dl_fs_exists() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("sample");
    let path_str = temp_file.path_str();

    let code = format!("return d.fs.exists('{path_str}')");
    let exists: bool = engine.lua.load(&code).eval().expect("Execution failed");
    assert!(exists);

    let not_exists: bool = engine
        .lua
        .load("return d.fs.exists('/tmp/non_existent_dale_test_file.tmp')")
        .eval()
        .expect("Execution failed");
    assert!(!not_exists);

    let nil_arg: bool = engine.lua.load("return d.fs.exists(nil)").eval().expect("Execution failed");
    assert!(!nil_arg);

    let empty_arg: bool = engine.lua.load("return d.fs.exists('')").eval().expect("Execution failed");
    assert!(!empty_arg);
}

/// Verifies that `d.fs.read` reads file contents, records dependencies, and returns `nil` for missing paths.
#[test]
fn test_dl_fs_read() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("test content");
    let path_str = temp_file.path_str();

    let code = format!("return d.fs.read('{path_str}')");
    let content: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(content, "test content");

    let deps_code = format!("d.fs.read('{path_str}')\nreturn __DALE_GET_DEPENDENCIES()");
    let deps: Vec<String> = engine.lua.load(&deps_code).eval().expect("Execution failed");
    assert!(deps.iter().any(|d| d.contains(&path_str)));

    let missing_code = "return d.fs.read('/tmp/non_existent_dale_test_file.tmp')";
    let missing_res: Option<String> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let nil_res: Option<String> = engine.lua.load("return d.fs.read(nil)").eval().expect("Execution failed");
    assert!(nil_res.is_none());
}

/// Verifies that `d.fs.read_lines` reads non-comment lines and returns an empty table `{}` for missing files.
#[test]
fn test_dl_fs_read_lines() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("first\n# comment\nsecond\n");
    let path_str = temp_file.path_str();

    let code = format!("return d.fs.read_lines('{path_str}')");
    let lines: Vec<String> = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "first");
    assert_eq!(lines[1], "second");

    let missing_code = "return d.fs.read_lines('/tmp/non_existent_dale_test_file.tmp')";
    let missing_lines: Table = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert_eq!(missing_lines.raw_len(), 0);
}

/// Verifies that `d.fs.read_json` parses JSON files, returns `nil` for missing paths, and errors on malformed syntax or wrong extensions.
#[test]
fn test_dl_fs_read_json() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let json_content = r#"{"artist":"Sample"}"#;
    let temp_file = TempFile::with_extension(json_content, "json");
    let path_str = temp_file.path_str();

    let code = format!("local t = d.fs.read_json('{path_str}')\nreturn t.artist");
    let artist: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(artist, "Sample");

    let missing_code = "return d.fs.read_json('/tmp/non_existent_dale_test_file.json')";
    let missing_res: Option<Table> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let bad_file = TempFile::with_extension("{invalid_json", "json");
    let bad_path = bad_file.path_str();
    let bad_code = format!("return d.fs.read_json('{bad_path}')");
    let bad_res = engine.lua.load(&bad_code).eval::<Table>();
    assert!(bad_res.is_err());

    let wrong_ext_file = TempFile::with_extension(json_content, "txt");
    let wrong_ext_path = wrong_ext_file.path_str();
    let wrong_ext_code = format!("return d.fs.read_json('{wrong_ext_path}')");
    let wrong_ext_res = engine.lua.load(&wrong_ext_code).eval::<Table>();
    assert!(wrong_ext_res.is_err());
}

/// Verifies that `d.fs.read_toml` parses TOML files, returns `nil` for missing paths, and errors on malformed syntax or wrong extensions.
#[test]
fn test_dl_fs_read_toml() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let toml_content = r#"[album]
artist = "Sample""#;
    let temp_file = TempFile::with_extension(toml_content, "toml");
    let path_str = temp_file.path_str();

    let code = format!("local t = d.fs.read_toml('{path_str}')\nreturn t.album.artist");
    let artist: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(artist, "Sample");

    let missing_code = "return d.fs.read_toml('/tmp/non_existent_dale_test_file.toml')";
    let missing_res: Option<Table> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let bad_file = TempFile::with_extension("[album", "toml");
    let bad_path = bad_file.path_str();
    let bad_code = format!("return d.fs.read_toml('{bad_path}')");
    let bad_res = engine.lua.load(&bad_code).eval::<Table>();
    assert!(bad_res.is_err());

    let wrong_ext_file = TempFile::with_extension(toml_content, "txt");
    let wrong_ext_path = wrong_ext_file.path_str();
    let wrong_ext_code = format!("return d.fs.read_toml('{wrong_ext_path}')");
    let wrong_ext_res = engine.lua.load(&wrong_ext_code).eval::<Table>();
    assert!(wrong_ext_res.is_err());
}
