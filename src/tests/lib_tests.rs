use mlua::{Lua, Table};

//extern crate luc;
use super::{get_lua_module, get_or_create_lua_module, make_lua_context};
//use luc::FromLuaConfig;

//#[derive(Clone, Debug, FromLuaConfig)]
//struct TestConfig {
//    value: i32,
//    label: String,
//    opt_label: Option<String>,
//}
//
#[test]
fn test_get_lua_module() {
    let lua = Lua::new();
    let module = get_lua_module(&lua, "io");

    assert!(module.is_some());
}

#[test]
fn test_get_lua_module_none() {
    let lua = Lua::new();
    let module = get_lua_module(&lua, "something");

    assert!(module.is_none());
}

#[test]
fn test_get_or_create_lua_module() {
    let lua = Lua::new();

    let module = get_lua_module(&lua, "something");
    assert!(module.is_none());

    let created = get_or_create_lua_module(&lua, "something");
    let module = get_lua_module(&lua, "something");

    assert!(created.is_some());
    assert!(module.is_some());
    assert_eq!(created.unwrap(), module.unwrap());
}

#[test]
fn test_make_lua_context() {
    let data_name = "uwu";
    let lua = make_lua_context(data_name, None).unwrap();
    let module = get_lua_module(&lua, data_name);

    assert!(module.is_some());
}

#[test]
fn test_make_lua_context_with_module() {
    let data_name = "uwu";
    let module_name = "hewwo";
    let lua = make_lua_context(data_name, Some(module_name)).unwrap();
    let module = get_lua_module(&lua, module_name);

    assert!(module.is_some());
}
