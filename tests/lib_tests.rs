use mlua::{Lua, Table};

extern crate clunk;
use clunk::FromLuaConfig;

#[derive(Clone, Debug, FromLuaConfig)]
struct TestConfig {
    pub value: i32,
    pub label: String,
    pub opt_label: Option<String>,
}

#[derive(Clone, Debug, FromLuaConfig)]
struct TestInner {
    pub label: String,
}

#[derive(Clone, Debug, FromLuaConfig)]
struct TestOuter {
    pub label: String,
    pub inner: TestInner,
}

// proc-macro panic
//struct TestArray(Vec<i32>);
#[derive(Clone, Debug, FromLuaConfig)]
struct TestArray {
    pub array: Vec<i32>,
}

#[test]
fn test_loaded_config() {
    let module_path = "tests/data/3.lua";
    let data_name = "thetest";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, data_name, None).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.table_name, data_name);
    assert_eq!(config.data.value, 3);
    assert_eq!(config.data.label, "oh hai");
}

#[test]
fn test_loaded_config_submodule() {
    let module_path = "tests/data/4.lua";
    let data_name = "stuff";
    let module_name = "thetest";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, data_name, Some(module_name)).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.table_name, data_name);
    assert_eq!(config.module_name.unwrap(), module_name);
    assert_eq!(config.data.value, 3);
    assert_eq!(config.data.label, "oh hai");
}

#[test]
fn test_loaded_config_submodule2() {
    let module_path = "tests/data/4.lua";
    let data_name = "stuff";
    let module_name = "thetest";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, data_name, Some(module_name)).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.module_name.unwrap(), module_name);
    assert_eq!(config.table_name, data_name);
    assert_eq!(config.data.value, 3);
    assert_eq!(config.data.label, "oh hai");
}

#[test]
fn test_loaded_config_inner() {
    let module_path = "tests/data/inner.lua";
    let table_name = "inner";
    let config: clunk::Clunk<TestInner> =
        clunk::Clunk::load(module_path, table_name, None).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.table_name, table_name);
    assert_eq!(config.data.label, "oh no");
}

#[test]
fn test_loaded_config_outer() {
    let module_path = "tests/data/outer.lua";
    let table_name = "outer";
    let config: clunk::Clunk<TestOuter> =
        clunk::Clunk::load(module_path, table_name, None).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.table_name, table_name);
    assert_eq!(config.data.label, "oh yes");
    assert_eq!(config.data.inner.label, "oh no");
}

#[test]
fn test_loaded_config_array() {
    let module_path = "tests/data/array.lua";
    let table_name = "array";
    let config: clunk::Clunk<TestArray> =
        clunk::Clunk::load(module_path, table_name, None).unwrap();

    assert_eq!(config.config_path, module_path);
    assert_eq!(config.table_name, table_name);
    assert_eq!(config.data.array.len(), 3);
}

#[test]
#[should_panic = "ConfigDataNotTable"]
fn test_loaded_config_data_not_table() {
    let module_path = "tests/data/bad0.lua";
    let module_name = "config";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, module_name, None).unwrap();
}

#[test]
#[should_panic = "TypeConversionFailed"]
fn test_loaded_config_incorrect_field_type() {
    let module_path = "tests/data/bad1.lua";
    let module_name = "config";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, module_name, None).unwrap();
}

#[test]
#[should_panic = "TableMissingField"]
fn test_loaded_config_module_not_table() {
    let module_path = "tests/data/bad2.lua";
    let module_name = "config";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, module_name, None).unwrap();
}

#[test]
#[should_panic = "TableMissingField"]
fn test_loaded_config_submodule_not_table() {
    let module_path = "tests/data/bad3.lua";
    let data_name = "settings";
    let module_name = "config";
    let config: clunk::Clunk<TestConfig> =
        clunk::Clunk::load(module_path, data_name, Some(module_name)).unwrap();
}
