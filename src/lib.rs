#![deny(elided_lifetimes_in_paths)]

/**
 * file: lib.rs
 * desc: main clunk library.
 */
pub use ::mlua;
use mlua::{Lua, Table, Value};
use std::path::Path;

pub use clunk_error::{ClunkError, ClunkResult};
pub use clunk_macro::FromLuaConfig;

/// Given some lua context, this will return the module with the given name.
/// The module is returned as a lua Table.
///
/// args
///   lua,         the lua context
///   module_name, lua module name we're looking for
///
fn get_lua_module<'lua>(lua: &'lua mlua::Lua, module_name: &str) -> Option<Table<'lua>> {
    let globals = lua.globals();
    let package: Table<'_> = globals.get("package").ok()?;
    let loaded: Table<'_> = package.get("loaded").ok()?;

    match loaded.get(module_name).ok()? {
        Value::Table(m) => Some(m),
        _ => None,
    }
}

/// Given some lua context, this will return the module with the given name or create and
/// return it if it doesn't exist. The module is returned as a lua Table.
///
/// args
///   lua,         the lua context
///   module_name, lua module name we're looking for
///
fn get_or_create_lua_module<'lua>(lua: &'lua mlua::Lua, module_name: &str) -> Option<Table<'lua>> {
    let globals = lua.globals();
    let package: Table<'_> = globals.get("package").ok()?;
    let loaded: Table<'_> = package.get("loaded").ok()?;

    match loaded.get(module_name).ok()? {
        Value::Table(m) => Some(m),
        Value::Nil => {
            let module = lua.create_table().ok()?;
            loaded.set(module_name, module.clone()).ok()?;
            Some(module)
        }
        _ => None,
    }
}

/// Generates a brand new lua context and sets it up to load data from the config.
///
/// args
///   table_name,  the name of the table in the lua script that contains the config data
///   module_name, an optional module within the lua script that contains the table (above) which
///                contains the config data
///
pub fn make_lua_context(table_name: &str, module_name: Option<&str>) -> ClunkResult<Lua> {
    let lua = Lua::new();

    {
        // no module is passed, so the config data and struct will be loaded directly from
        // <table_name>, meaning that in rust `struct MyConfig {..}` will be loaded directly from
        // `package.loaded.<table_name>` in lua
        let module = if module_name.is_none() {
            //
            match get_or_create_lua_module(&lua, table_name) {
                Some(m) => m,
                // None is returned if the module exists but is not a table (or we couldn't create
                // the module for some unrelated reason)
                None => {
                    return Err(ClunkError::LoadedModuleNotTable(table_name.to_string()));
                }
            }
        } else {
            // todo: setters/getters for stuff in <module_name>?
            // otherwise, we'll create a module that will contain the config table in
            // addition to any other functions, constants, etc. created by the application.
            // this means that in rust `struct MyConfig {..}` will be loaded from
            // `package.loaded.<module_name>.<table_name>` in lua
            let module_name = module_name.unwrap();

            // create the module that will contain everything
            match get_or_create_lua_module(&lua, module_name) {
                Some(module) => {
                    // now create the table to model config data
                    let data_table = lua.create_table()?;
                    module.set(table_name, data_table.clone())?;
                    module
                }
                // same comment as above wrt to returning None
                None => {
                    return Err(ClunkError::LoadedModuleNotTable(module_name.to_string()));
                }
            }
        };
    }

    Ok(lua)
}

/// Adds a lua function named `config_filepath` to the current lua context. This function can be
/// used to return the filepath of the current config file. This allows the config to be aware of
/// its own filepath.
///
/// args
///   lua,         the lua context
///   module_name, lua module name which contains the config data
///   config_path, the filepath to the config file
///

//impl<'lua> Table<'lua>
//pub fn set<K, V>(&self, key: K, value: V) -> Result<()>
//where
//    K: IntoLua<'lua>,
//    V: IntoLua<'lua>,
fn add_filepath_module_function(
    lua: &Lua,
    module_name: &str,
    config_path: &str,
) -> Result<(), mlua::Error> {
    let globals = lua.globals();
    let package: Table<'_> = globals.get("package")?;
    let loaded: Table<'_> = package.get("loaded")?;
    let module = get_lua_module(lua, module_name).unwrap();

    module.set("config_filepath", config_path)?;

    Ok(())
}

/// A clunk represents a loaded config file. Contains all the necessary metadata and the required
/// lua context for the config.
///
#[derive(Debug)]
pub struct Clunk<T> {
    /// filepath to the config
    pub config_path: String,
    /// custom lua module name we're using as a namespace for the config
    pub table_name: String,
    /// submodule within our custom lua module that contains the actual config data
    pub module_name: Option<String>,
    /// lua context
    lua: Option<mlua::Lua>,
    /// the config data
    pub data: T,
}

// todo: better error messages when incorrect syntax is used
/// the clunk impl. whatever the generic config type is, it must implemente mlua::FromLua.
/// there's a proc macro in this crate that will do exactly that.
///
impl<T> Clunk<T>
where
    T: Clone + for<'l> mlua::FromLua<'l>,
{
    /// Loads a config file and returns a clunk with the config data.
    ///
    /// args
    ///  config_path, config filepath
    ///  table_name,  name of the table in the lua script that contains the config data
    ///  module_name, an optional module within the lua script that contains the table (above)
    ///               which contains the config data
    pub fn load(
        config_path: impl AsRef<Path>,
        table_name: &str,
        module_name: Option<&str>,
    ) -> Result<Self, ClunkError> {
        let lua = make_lua_context(table_name, module_name)?;

        if module_name.is_some() {
            add_filepath_module_function(
                &lua,
                module_name.unwrap(),
                config_path.as_ref().to_str().unwrap(),
            )?;
        } else {
            add_filepath_module_function(&lua, table_name, config_path.as_ref().to_str().unwrap())?;
        }

        lua
            .load(std::fs::read_to_string(&config_path)?.as_str())
            .exec()?;

        //match loaded.get(module_name).ok()? {
        //Value::Table(m) => Some(m),
        //_ => None,
        let cfg = {
            let globals = lua.globals();

            let cfg = if module_name.is_some() {
                get_lua_module(&lua, module_name.unwrap())
                    .unwrap()
                    .get::<_, T>(table_name)
                //.unwrap()
            } else {
                globals.get::<_, T>(table_name)
            };

            match cfg {
                Ok(c) => c,
                Err(e) => {
                    return Err(ClunkError::from(e));
                }
            }
        };

        Ok(Self {
            config_path: std::path::PathBuf::from(config_path.as_ref())
                .into_os_string()
                .into_string()
                .unwrap(),
            table_name: table_name.to_string(),
            module_name: module_name.map(|s| s.to_string()),
            lua: Some(lua),
            //data: Ok(cfg),
            data: cfg,
        })
    }
}

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod lib_tests;
