use mlua::{Lua, Table};

//extern crate luc;
use super::{get_lua_module, get_or_create_lua_module, make_lua_context};
//use luc::FromLuaConfig;

#[derive(Clone, Debug, crate::FromLuaConfig)]
struct TestConfigWithIgnore {
    msg: String,
    id: u32,
    #[ignore_field]
    ignored_field: String,
    #[ignore_field]
    ignored_number: i32,
}

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

#[test]
fn test_ignore_attribute() {
    use std::fs;

    // Create a temporary lua config file for testing (global config)
    let lua_content = r#"config = {
    msg = "hello from test",
    id = 123
    -- ignored_field and ignored_number are not defined
}"#;

    fs::write("test_ignore_temp.lua", lua_content).unwrap();

    // Test that we can load the config even with missing fields that are ignored
    let config: crate::Clunk<TestConfigWithIgnore> =
        crate::Clunk::load("test_ignore_temp.lua", "config", None).unwrap();

    // Verify normal fields are loaded correctly
    assert_eq!(config.data.msg, "hello from test");
    assert_eq!(config.data.id, 123);

    // Verify ignored fields use default values
    assert_eq!(config.data.ignored_field, String::default());
    assert_eq!(config.data.ignored_number, i32::default());

    // Clean up
    fs::remove_file("test_ignore_temp.lua").unwrap();
}
