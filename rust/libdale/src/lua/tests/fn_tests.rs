//! Unit tests for Lua function utilities (`dl.fn`).

use crate::lua::LuaEngine;

/// Verifies that `dl.fn.require` returns the value when given valid input.
#[test]
fn test_dl_fn_require_valid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result: String = engine
        .lua
        .load("return dl.fn.require('hello')")
        .eval()
        .expect("Execution failed");

    assert_eq!(result, "hello");
}

/// Verifies that `dl.fn.require` fails when given empty input.
#[test]
fn test_dl_fn_require_invalid() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");
    let result = engine.lua.load("return dl.fn.require('')").eval::<String>();

    assert!(result.is_err());
}

/// Verifies that `dl.fn.type_check` validates data types correctly.
#[test]
fn test_dl_fn_type_check() {
    let engine = LuaEngine::new().expect("Failed to create LuaEngine");

    let valid_str: String = engine
        .lua
        .load("return dl.fn.type_check('sample', 'string')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_str, "sample");

    let valid_num: f64 = engine
        .lua
        .load("return dl.fn.type_check(42, 'number')")
        .eval()
        .expect("Execution failed");
    assert_eq!(valid_num, 42.0);

    let invalid = engine
        .lua
        .load("return dl.fn.type_check(123, 'string')")
        .eval::<String>();
    assert!(invalid.is_err());
}
