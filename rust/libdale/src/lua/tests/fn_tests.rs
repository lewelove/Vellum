//! Unit tests for Lua function utilities (`d.fn`).

use crate::lua::LuaEngine;
use mlua::Table;

/// Verifies that `d.fn.present` returns the value when given valid input.
#[test]
fn test_dl_fn_present_valid() {
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
fn test_dl_fn_present_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return d.fn.present('')").eval::<String>();
    assert!(result.is_err());

    let empty_table_res = engine.lua.load("return d.fn.present({})").eval::<Table>();
    assert!(empty_table_res.is_err());
}

/// Verifies that `d.fn.type_check` validates data types correctly and allows missing values.
#[test]
fn test_dl_fn_type_check() {
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
fn test_dl_fn_coalesce() {
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
