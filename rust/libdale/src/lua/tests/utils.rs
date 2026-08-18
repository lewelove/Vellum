//! Unit tests for Lua utility modules (`d.fn`, `d.fs`, `d.get`, `d.json`, `d.toml`).

use super::TempFile;
use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `d.fn.present` returns the value when given valid input.
#[test]
fn test_d_fn_present_valid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result: String = engine
        .lua
        .load("return d.fn.present('hello')")
        .eval()
        .expect("Execution failed");
    assert_eq!(result, "hello");
}

/// Verifies that `d.fn.present` fails when given empty input or an empty table.
#[test]
fn test_d_fn_present_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.fn.present('')").eval::<String>();
    assert!(result.is_err());

    let empty_table_res = engine.lua.load("return d.fn.present({})").eval::<Table>();
    assert!(empty_table_res.is_err());
}

/// Verifies that `d.fn.type_check` validates data types correctly and allows missing values.
#[test]
fn test_d_fn_type_check() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let valid_str: String = engine
        .lua
        .load("return d.fn.type_check('sample', 'string')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_str, "sample");

    let valid_num: f64 = engine
        .lua
        .load("return d.fn.type_check(42, 'number')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_num, 42.0);

    let invalid = engine
        .lua
        .load("return d.fn.type_check(123, 'string')")
        .eval::<String>();
    assert!(invalid.is_err());

    let missing: Option<String> = engine
        .lua
        .load("return d.fn.type_check(nil, 'string')")
        .eval()
        .expect("Execution failed");
    assert!(missing.is_none());
}

/// Verifies that `d.fn.coalesce` returns the first non-empty argument.
#[test]
fn test_d_fn_coalesce() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let first_valid: String = engine
        .lua
        .load("return d.fn.coalesce('', nil, 'first', 'second')")
        .eval()
        .expect("Execution failed");
    assert_eq!(first_valid, "first");

    let table_valid: String = engine
        .lua
        .load("return d.fn.coalesce({}, '', 'from_table')")
        .eval()
        .expect("Execution failed");
    assert_eq!(table_valid, "from_table");

    let all_empty: Option<String> = engine
        .lua
        .load("return d.fn.coalesce('', nil, {})")
        .eval()
        .expect("Execution failed");
    assert!(all_empty.is_none());
}

/// Verifies that `d.get` performs safe table traversal across dot paths, variable arguments, array indices, and missing keys.
#[test]
fn test_d_get() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let test_setup = r#"
        local sample = {
            metadata = {
                album = {
                    title = "Selected Ambient Works",
                    artist = "Aphex Twin"
                },
                tracks = {
                    { title = "Xtal", duration = 294 },
                    { title = "Tha", duration = 546 }
                }
            }
        }
    "#;

    let dot_path_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.album.title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(dot_path_res, "Selected Ambient Works");

    let varargs_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata', 'album', 'artist')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(varargs_res, "Aphex Twin");

    let mixed_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.album', 'title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(mixed_res, "Selected Ambient Works");

    let array_index_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.tracks', 2, 'title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(array_index_res, "Tha");

    let array_dot_res: u32 = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.tracks.1.duration')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(array_dot_res, 294);

    let missing_path: Option<String> = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.nonexistent.field')"))
        .eval()
        .expect("Execution failed");
    assert!(missing_path.is_none());

    let nil_target: Option<String> = engine
        .lua
        .load("return d.get(nil, 'metadata.album.title')")
        .eval()
        .expect("Execution failed");
    assert!(nil_target.is_none());
}

/// Verifies that `d.fs.exists` reports file existence correctly and returns `false` on missing or nil paths.
#[test]
fn test_d_fs_exists() {
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
fn test_d_fs_read() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file = TempFile::new("test content");
    let path_str = temp_file.path_str();

    let code = format!("return d.fs.read('{path_str}')");
    let content: String = engine.lua.load(&code).eval().expect("Execution failed");
    assert_eq!(content, "test content");

    let deps = engine
        .lua
        .app_data_ref::<crate::lua::EngineContext>()
        .unwrap()
        .take_dependencies();
    assert!(deps.iter().any(|d| d.to_string_lossy().contains(&path_str)));

    let missing_code = "return d.fs.read('/tmp/non_existent_dale_test_file.tmp')";
    let missing_res: Option<String> = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert!(missing_res.is_none());

    let nil_res: Option<String> = engine.lua.load("return d.fs.read(nil)").eval().expect("Execution failed");
    assert!(nil_res.is_none());
}

/// Verifies that `d.fs.read_lines` reads non-comment lines and returns an empty table `{}` for missing files.
#[test]
fn test_d_fs_read_lines() {
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
fn test_d_fs_read_json() {
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
fn test_d_fs_read_toml() {
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

/// Verifies that `d.json.decode` and `d.json.encode` convert data structures correctly.
#[test]
fn test_d_json_encode_decode() {
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
fn test_d_json_decode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.json.decode('{bad_json:')").eval::<Table>();
    assert!(result.is_err());
}

/// Verifies that `d.json.encode` fails when given non-serializable Lua values.
#[test]
fn test_d_json_encode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.json.encode({ func = function() end })").eval::<String>();
    assert!(result.is_err());
}

/// Verifies that `d.toml.decode` and `d.toml.encode` convert data structures correctly.
#[test]
fn test_d_toml_encode_decode() {
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
fn test_d_toml_decode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.toml.decode('[album')").eval::<Table>();
    assert!(result.is_err());
}

/// Verifies that `d.toml.encode` fails when given non-serializable Lua values.
#[test]
fn test_d_toml_encode_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.toml.encode({ func = function() end })").eval::<String>();
    assert!(result.is_err());
}
