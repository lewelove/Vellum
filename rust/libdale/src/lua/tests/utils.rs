use super::TempFile;
use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `d.fn.present` returns the value when given valid, non-empty input.
#[test]
fn test_d_fn_present_valid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result: String = engine
        .lua
        .load("return d.fn.present('hello')")
        .eval()
        .expect("Execution failed");
    assert_eq!(result, "hello");

    let num_res: f64 = engine
        .lua
        .load("return d.fn.present(123)")
        .eval()
        .expect("Execution failed");
    assert_eq!(num_res, 123.0);
}

/// Verifies that `d.fn.present` raises an error when given an empty string, nil, whitespace-only string, or empty table.
#[test]
fn test_d_fn_present_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Empty string rejection
    let empty_str_res = engine.lua.load("return d.fn.present('')").eval::<String>();
    assert!(empty_str_res.is_err());

    // Whitespace string rejection
    let ws_res = engine.lua.load("return d.fn.present('   ')").eval::<String>();
    assert!(ws_res.is_err());

    // Nil rejection
    let nil_res = engine.lua.load("return d.fn.present(nil)").eval::<String>();
    assert!(nil_res.is_err());

    // Empty table rejection
    let empty_table_res = engine.lua.load("return d.fn.present({})").eval::<Table>();
    assert!(empty_table_res.is_err());
}

/// Verifies that `d.fn.type_check` validates data types correctly and allows missing or empty values.
#[test]
fn test_d_fn_type_check() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Valid string types
    let valid_str: String = engine
        .lua
        .load("return d.fn.type_check('sample', 'string')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_str, "sample");

    // Valid number types
    let valid_num: f64 = engine
        .lua
        .load("return d.fn.type_check(42, 'number')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_num, 42.0);

    // Valid boolean types
    let valid_bool: bool = engine
        .lua
        .load("return d.fn.type_check(true, 'boolean')")
        .eval()
        .expect("Execution failed");
    assert!(valid_bool);

    // Valid table/array types
    let valid_array: Table = engine
        .lua
        .load("return d.fn.type_check({1, 2}, 'array')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_array.raw_len(), 2);

    // Invalid type mismatches raise errors
    let invalid_num = engine
        .lua
        .load("return d.fn.type_check(123, 'string')")
        .eval::<String>();
    assert!(invalid_num.is_err());

    let invalid_str = engine
        .lua
        .load("return d.fn.type_check('abc', 'number')")
        .eval::<f64>();
    assert!(invalid_str.is_err());

    // Nil and empty string return nil without error
    let missing_nil: Option<String> = engine
        .lua
        .load("return d.fn.type_check(nil, 'string')")
        .eval()
        .expect("Execution failed");
    assert!(missing_nil.is_none());

    let missing_empty: Option<String> = engine
        .lua
        .load("return d.fn.type_check('', 'string')")
        .eval()
        .expect("Execution failed");
    assert!(missing_empty.is_none());
}

/// Verifies that `d.fn.coalesce` returns the first non-empty, non-nil argument.
#[test]
fn test_d_fn_coalesce() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Skips empty string and nil
    let first_valid: String = engine
        .lua
        .load("return d.fn.coalesce('', nil, 'first', 'second')")
        .eval()
        .expect("Execution failed");
    assert_eq!(first_valid, "first");

    // Skips empty tables
    let table_valid: String = engine
        .lua
        .load("return d.fn.coalesce({}, '', 'from_table')")
        .eval()
        .expect("Execution failed");
    assert_eq!(table_valid, "from_table");

    // All empty arguments return nil
    let all_empty: Option<String> = engine
        .lua
        .load("return d.fn.coalesce('', nil, {})")
        .eval()
        .expect("Execution failed");
    assert!(all_empty.is_none());
}

/// Verifies that `d.get` performs safe table traversal across dot paths, varargs, array indices, and missing keys.
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

    // Dot path lookup
    let dot_path_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.album.title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(dot_path_res, "Selected Ambient Works");

    // Varargs lookup
    let varargs_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata', 'album', 'artist')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(varargs_res, "Aphex Twin");

    // Mixed path and segment lookup
    let mixed_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.album', 'title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(mixed_res, "Selected Ambient Works");

    // Array numeric index lookup
    let array_index_res: String = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.tracks', 2, 'title')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(array_index_res, "Tha");

    // Array string index in dot path
    let array_dot_res: u32 = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.tracks.1.duration')"))
        .eval()
        .expect("Execution failed");
    assert_eq!(array_dot_res, 294);

    // Non-existent field returns nil
    let missing_path: Option<String> = engine
        .lua
        .load(format!("{test_setup}\nreturn d.get(sample, 'metadata.nonexistent.field')"))
        .eval()
        .expect("Execution failed");
    assert!(missing_path.is_none());

    // Nil target returns nil
    let nil_target: Option<String> = engine
        .lua
        .load("return d.get(nil, 'metadata.album.title')")
        .eval()
        .expect("Execution failed");
    assert!(nil_target.is_none());
}

/// Verifies that `d.fs.basename` extracts the terminal path component correctly across file paths, directory paths, and root.
#[test]
fn test_d_fs_basename() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Standard filename extraction
    let file_res: String = engine
        .lua
        .load("return d.fs.basename('/path/to/file.txt')")
        .eval()
        .expect("Execution failed");
    assert_eq!(file_res, "file.txt");

    // Trailing slash path extraction
    let dir_res: String = engine
        .lua
        .load("return d.fs.basename('/path/to/directory/')")
        .eval()
        .expect("Execution failed");
    assert_eq!(dir_res, "directory");

    // Redundant trailing slashes
    let multi_slash_res: String = engine
        .lua
        .load("return d.fs.basename('/path/to/nested///')")
        .eval()
        .expect("Execution failed");
    assert_eq!(multi_slash_res, "nested");

    // Root directory returns empty string
    let root_res: String = engine
        .lua
        .load("return d.fs.basename('/')")
        .eval()
        .expect("Execution failed");
    assert_eq!(root_res, "");

    // Nil and empty input return nil
    let nil_res: Option<String> = engine
        .lua
        .load("return d.fs.basename(nil)")
        .eval()
        .expect("Execution failed");
    assert!(nil_res.is_none());

    let empty_res: Option<String> = engine
        .lua
        .load("return d.fs.basename('')")
        .eval()
        .expect("Execution failed");
    assert!(empty_res.is_none());
}

/// Verifies that `d.fs.dirname` extracts directory prefixes and handles root and single-segment boundaries.
#[test]
fn test_d_fs_dirname() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Standard directory extraction
    let dir_res: String = engine
        .lua
        .load("return d.fs.dirname('/path/to/file.txt')")
        .eval()
        .expect("Execution failed");
    assert_eq!(dir_res, "/path/to");

    // Trailing slash directory extraction
    let trailing_res: String = engine
        .lua
        .load("return d.fs.dirname('/path/to/directory/')")
        .eval()
        .expect("Execution failed");
    assert_eq!(trailing_res, "/path/to");

    // Single root child returns root slash
    let root_elem_res: String = engine
        .lua
        .load("return d.fs.dirname('/file.txt')")
        .eval()
        .expect("Execution failed");
    assert_eq!(root_elem_res, "/");

    // Root path returns root slash
    let root_res: String = engine
        .lua
        .load("return d.fs.dirname('/')")
        .eval()
        .expect("Execution failed");
    assert_eq!(root_res, "/");

    // Multiple root slashes return root slash
    let multi_root_res: String = engine
        .lua
        .load("return d.fs.dirname('///')")
        .eval()
        .expect("Execution failed");
    assert_eq!(multi_root_res, "/");

    // Relative single element has no dirname
    let rel_single: Option<String> = engine
        .lua
        .load("return d.fs.dirname('file.txt')")
        .eval()
        .expect("Execution failed");
    assert!(rel_single.is_none());

    // Nil and empty input return nil
    let nil_res: Option<String> = engine
        .lua
        .load("return d.fs.dirname(nil)")
        .eval()
        .expect("Execution failed");
    assert!(nil_res.is_none());

    let empty_res: Option<String> = engine
        .lua
        .load("return d.fs.dirname('')")
        .eval()
        .expect("Execution failed");
    assert!(empty_res.is_none());
}

/// Verifies that `d.fs.joinpath` concatenates and normalizes paths with redundant slashes and empty segments.
#[test]
fn test_d_fs_joinpath() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let joined: String = engine
        .lua
        .load("return d.fs.joinpath('/music', 'Artist - Title [FLAC]', 'cover.jpg')")
        .eval()
        .expect("Execution failed");
    assert_eq!(joined, "/music/Artist - Title [FLAC]/cover.jpg");

    // Collapses redundant slashes
    let redundant: String = engine
        .lua
        .load("return d.fs.joinpath('/music/', '//Artist/', '/01.flac')")
        .eval()
        .expect("Execution failed");
    assert_eq!(redundant, "/music/Artist/01.flac");

    // Ignores empty or nil parts
    let with_empty: String = engine
        .lua
        .load("return d.fs.joinpath('/music', '', 'Artist', nil, '02.flac')")
        .eval()
        .expect("Execution failed");
    assert_eq!(with_empty, "/music/Artist/02.flac");
}

/// Verifies that `d.fs.normalize` resolves relative components, backslashes, and home directory prefixes.
#[test]
fn test_d_fs_normalize() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    // Resolves dot and double-dot segments
    let resolved: String = engine
        .lua
        .load("return d.fs.normalize('/a/b/../c/./d')")
        .eval()
        .expect("Execution failed");
    assert_eq!(resolved, "/a/c/d");

    // Converts backslashes to forward slashes
    let backslashes: String = engine
        .lua
        .load(r#"return d.fs.normalize("C:\\music\\album\\track.flac")"#)
        .eval()
        .expect("Execution failed");
    assert_eq!(backslashes, "C:/music/album/track.flac");

    // Empty and root normalizations
    let empty_res: String = engine
        .lua
        .load("return d.fs.normalize('')")
        .eval()
        .expect("Execution failed");
    assert_eq!(empty_res, "");

    let root_res: String = engine
        .lua
        .load("return d.fs.normalize('/')")
        .eval()
        .expect("Execution failed");
    assert_eq!(root_res, "/");
}

/// Verifies that `d.fs.parents` iterates upward through all directory ancestors up to root.
#[test]
fn test_d_fs_parents() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let code = r#"
        local parents = {}
        for p in d.fs.parents('/a/b/c/file.txt') do
            table.insert(parents, p)
        end
        return parents
    "#;
    let parents: Vec<String> = engine.lua.load(code).eval().expect("Execution failed");
    assert_eq!(parents, vec!["/a/b/c", "/a/b", "/a", "/"]);
}

/// Verifies that `d.fs.dir` enumerates children at single and recursive depths without relative path truncation.
#[test]
fn test_d_fs_dir() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_file1 = TempFile::with_extension("content1", "lrc");
    let temp_file2 = TempFile::with_extension("content2", "txt");
    let parent = temp_file1.0.parent().unwrap();
    let parent_str = parent.to_string_lossy();

    // Depth = 1 scanning
    let code_depth1 = format!(
        r#"
        local names = {{}}
        for name, type_ in d.fs.dir('{parent_str}', {{ depth = 1 }}) do
            if type_ == 'file' and name:find('^dale_test_') then
                table.insert(names, name)
            end
        end
        return names
    "#
    );
    let names_depth1: Vec<String> = engine.lua.load(&code_depth1).eval().expect("Execution failed");
    assert!(names_depth1.len() >= 2);

    // Depth > 1 recursive scanning retains full relative path without string corruption
    let code_recursive = format!(
        r#"
        local rel_paths = {{}}
        for rel, type_ in d.fs.dir('{parent_str}', {{ depth = 3 }}) do
            if type_ == 'file' and rel:find('dale_test_') then
                table.insert(rel_paths, rel)
            end
        end
        return rel_paths
    "#
    );
    let rel_paths: Vec<String> = engine.lua.load(&code_recursive).eval().expect("Execution failed");
    assert!(rel_paths.iter().all(|p| !p.starts_with('/')));

    // Skip callback verification
    let code_skip = format!(
        r#"
        local visited = {{}}
        for rel, _ in d.fs.dir('{parent_str}', {{
            depth = 3,
            skip = function(dir_rel)
                return true
            end
        }}) do
            table.insert(visited, rel)
        end
        return visited
    "#
    );
    let visited: Vec<String> = engine.lua.load(&code_skip).eval().expect("Execution failed");
    assert!(!visited.is_empty());

    let _ = temp_file2;
}

/// Verifies that `d.fs.find` locates items using exact names, name tables, and predicate functions with consistent (name, full_path) arguments.
#[test]
fn test_d_fs_find() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_lrc = TempFile::with_extension("lyrics", "lrc");
    let parent = temp_lrc.0.parent().unwrap();
    let parent_str = parent.to_string_lossy();
    let file_name = temp_lrc.0.file_name().unwrap().to_string_lossy().to_string();

    // Exact string match
    let code_exact = format!(
        r#"return d.fs.find('{file_name}', {{ path = '{parent_str}', type = 'file' }})"#
    );
    let exact_res: Vec<String> = engine.lua.load(&code_exact).eval().expect("Execution failed");
    assert_eq!(exact_res.len(), 1);
    assert_eq!(exact_res[0], temp_lrc.path_str());

    // Name table match
    let code_table = format!(
        r#"return d.fs.find({{ '{file_name}', 'nonexistent.txt' }}, {{ path = '{parent_str}', type = 'file' }})"#
    );
    let table_res: Vec<String> = engine.lua.load(&code_table).eval().expect("Execution failed");
    assert_eq!(table_res.len(), 1);
    assert_eq!(table_res[0], temp_lrc.path_str());

    // Downward predicate receives (name, full_path)
    let code_pred = format!(
        r#"
        return d.fs.find(function(name, path)
            return name == '{file_name}' and path:find('{file_name}$') ~= nil
        end, {{ path = '{parent_str}', limit = 1 }})
    "#
    );
    let pred_res: Vec<String> = engine.lua.load(&code_pred).eval().expect("Execution failed");
    assert_eq!(pred_res.len(), 1);
    assert_eq!(pred_res[0], temp_lrc.path_str());

    // Upward predicate consistently receives (name, full_path)
    let code_pred_up = format!(
        r#"
        return d.fs.find(function(name, path)
            return name == '{file_name}' and path:find('{file_name}$') ~= nil
        end, {{ path = '{parent_str}', upward = true, limit = 1 }})
    "#
    );
    let pred_up_res: Vec<String> = engine.lua.load(&code_pred_up).eval().expect("Execution failed");
    assert_eq!(pred_up_res.len(), 1);
    assert_eq!(pred_up_res[0], temp_lrc.path_str());
}

/// Verifies that `d.fs.root` locates containing project or album roots by ascending directory hierarchies.
#[test]
fn test_d_fs_root() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let temp_marker = TempFile::with_extension("marker", "toml");
    let parent = temp_marker.0.parent().unwrap();
    let parent_str = parent.to_string_lossy();
    let marker_name = temp_marker.0.file_name().unwrap().to_string_lossy().to_string();

    let code = format!(r#"return d.fs.root('{parent_str}', '{marker_name}')"#);
    let root_dir: Option<String> = engine.lua.load(&code).eval().expect("Execution failed");
    assert!(root_dir.is_some());
    assert_eq!(root_dir.unwrap(), parent_str);

    // Missing marker returns nil
    let missing_code = format!(r#"return d.fs.root('{parent_str}', 'non_existent_marker_file.toml')"#);
    let missing_root: Option<String> = engine.lua.load(&missing_code).eval().expect("Execution failed");
    assert!(missing_root.is_none());
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

/// Verifies that `d.fs.read` reads full file contents, records file dependencies in EngineContext, and returns nil for missing files.
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

/// Verifies that `d.fs.read_lines` reads non-comment lines with index/key bidirectional mapping.
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

    // Bidirectional lookup check
    let lookup_code = format!(
        r#"
        local t = d.fs.read_lines('{path_str}')
        return {{ t[1], t['first'], t[2], t['second'] }}
    "#
    );
    let lookup: Vec<mlua::Value> = engine.lua.load(&lookup_code).eval().expect("Execution failed");
    assert_eq!(lookup.len(), 4);

    let missing_code = "return d.fs.read_lines('/tmp/non_existent_dale_test_file.tmp')";
    let missing_lines: Table = engine.lua.load(missing_code).eval().expect("Execution failed");
    assert_eq!(missing_lines.raw_len(), 0);
}

/// Verifies that `d.fs.read_json` parses JSON files and enforces `.json` extensions.
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

/// Verifies that `d.fs.read_toml` parses TOML files and enforces `.toml` extensions.
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

/// Verifies that `d.json.decode` and `d.json.encode` convert data structures accurately.
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

/// Verifies that `d.json.decode` fails on malformed JSON.
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

/// Verifies that `d.toml.decode` and `d.toml.encode` convert data structures accurately.
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

/// Verifies that `d.toml.decode` fails on malformed TOML syntax.
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
