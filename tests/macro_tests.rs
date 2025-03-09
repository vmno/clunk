/**
 * file: macro_tests.rs
 * desc: tests for the procedural macro defined in src/macro.
 */
use mlua::Lua;

extern crate clunk;
use clunk::FromLuaConfig;

#[derive(Clone, Debug, FromLuaConfig)]
struct TestConfig {
    pub value: i32,
    pub label: String,
    pub opt_label: Option<String>,
}

#[test]
fn test_from_lua_config_macro() {
    let lua = Lua::new();
    let name = "test_config";
    let globals = lua.globals();

    lua.load(
        r#"
        test_config = {
            value = 42,
            label = "foobar",
            opt_label = "optional label",
        }
        "#,
    )
    .exec()
    .unwrap();

    let tc = globals.get::<_, TestConfig>(name).unwrap();

    assert_eq!(tc.value, 42);
    assert_eq!(tc.label, "foobar");
    assert_eq!(tc.opt_label, Some("optional label".to_string()));
}

// currently any extra fields in the lua table are just ignored, at some point we may want this
// behavior to be different/configurable
#[test]
fn test_from_lua_config_macro_extra_fields() {
    let lua = Lua::new();
    let name = "test_config";
    let globals = lua.globals();

    lua.load(
        r#"
        test_config = {
            value = 42,
            label = "foobar",
            opt_label = "optional label",
            extra_thing = "hewwo"
        }
        "#,
    )
    .exec()
    .unwrap();

    let tc = globals.get::<_, TestConfig>(name).unwrap();

    assert_eq!(tc.value, 42);
    assert_eq!(tc.label, "foobar");
    assert_eq!(tc.opt_label, Some("optional label".to_string()));
}

#[test]
fn test_from_lua_config_macro_no_optional() {
    let lua = Lua::new();
    let name = "test_config";
    let globals = lua.globals();

    lua.load(
        r#"
        test_config = {
            value = 42,
            label = "foobar",
        }
        "#,
    )
    .exec()
    .unwrap();

    let tc = globals.get::<_, TestConfig>(name).unwrap();

    assert_eq!(tc.value, 42);
    assert_eq!(tc.label, "foobar");
    assert_eq!(tc.opt_label, None);
}

#[test]
#[should_panic = "table is missing field"]
fn test_from_lua_config_macro_no_required() {
    let lua = Lua::new();
    let name = "test_config";
    let globals = lua.globals();

    lua.load(
        r#"
        test_config = {
            label = "foobar",
            opt_label = "optional label",
        }
        "#,
    )
    .exec()
    .unwrap();

    let tc = globals.get::<_, TestConfig>(name).unwrap();

    assert_eq!(tc.label, "foobar");
    assert_eq!(tc.opt_label, None);
}

#[test]
#[should_panic = "the wrong type"]
fn test_from_lua_config_macro_wrong_type() {
    let lua = Lua::new();
    let name = "test_config";
    let globals = lua.globals();

    lua.load(
        r#"
        test_config = {
            value = "42a",
            label = "foobar",
            opt_label = "optional label",
        }
        "#,
    )
    .exec()
    .unwrap();

    let tc = globals.get::<_, TestConfig>(name).unwrap();

    assert_eq!(tc.value, 42);
    assert_eq!(tc.label, "foobar");
}
