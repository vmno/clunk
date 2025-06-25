#![feature(trace_macros)]
//
use mlua::{chunk, Function, Lua, MetaMethod, Table, UserData, UserDataMethods, Value, Variadic};

use clunk;

fn eg0() {
    let lua = Lua::new();
    lua.load(std::fs::read_to_string("0.lua").unwrap().as_str())
        .exec()
        .unwrap();
    let globals = lua.globals();
    let tbl = globals.get::<_, Table>("config").unwrap();
    let msg = tbl.get::<_, String>("msg").unwrap();

    println!("config: {:?}", tbl);
    println!("config: {:?}", msg);
}

fn eg1() {
    let lua = Lua::new();
    let globals = lua.globals();

    let cfg_table = lua.create_table().unwrap();
    cfg_table.set("msg", "hello").unwrap();

    globals.set("config", cfg_table).unwrap();

    lua.load(std::fs::read_to_string("1.lua").unwrap().as_str())
        .exec()
        .unwrap();

    let tbl = globals.get::<_, Table>("config").unwrap();
    let msg = tbl.get::<_, String>("msg").unwrap();

    println!("config: {:?}", tbl);
    println!("config.msg: {:?}", msg);
}

// fromlua needs lifetime param
#[derive(Clone, Debug)]
struct EgData0 {
    msg: String,
    id: Option<u32>,
}

impl UserData for EgData0 {}

/*
impl FromLua<'_> for EgData0 {
    fn from_lua(value: Value, lua: &Lua) -> Result<Self> {
        let tbl = value.as_table().unwrap();
        let msg = tbl.get::<_, String>("msg")?;
        //let id = tbl.get::<_, u32>("id")?;
        //Ok(EgData0 { msg, id })
        let id = tbl.get::<_, Option<u32>>("id")?;
        Ok(EgData0 { msg, id: id })
    }
}
*/

fn eg2() {
    let lua = Lua::new();
    let globals = lua.globals();

    let cfg_table = lua.create_table().unwrap();
    cfg_table.set("msg", "hello").unwrap();

    globals.set("config", cfg_table).unwrap();

    lua.load(std::fs::read_to_string("2.lua").unwrap().as_str())
        .exec()
        .unwrap();

    let tbl = globals.get::<_, Table>("config").unwrap();
    let msg = tbl.get::<_, String>("msg").unwrap();

    // as userdata
    //   let ud = globals.get::<_, EgData0>("config").unwrap();

    println!("config: {:?}", tbl);
    println!("config.msg: {:?}", msg);
    println!("---");
    //println!("config.ud: {:?}", ud);
    //println!("config.ud.msg: {:?}", ud.msg);
    //println!("config.ud.id: {:?}", ud.id);
}

pub fn create_lua_module<'lua>(
    lua: &'lua Lua,
    module_name: &str,
) -> Result<Table<'lua>, mlua::Error> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let loaded: Table = package.get("loaded")?;

    match loaded.get(module_name) {
        Ok(module) => match module {
            // module doesn't exist, so create it
            Value::Nil => {
                let module = lua.create_table()?;
                loaded.set(module_name, module.clone())?;
                Ok(module)
            }
            // module exists so return it
            Value::Table(m) => Ok(m),
            // idk
            _ => Err(mlua::Error::RuntimeError(format!(
                "package.loaded.{} exists and is not a table",
                module_name
            ))),
        },
        Err(_) => {
            let module = lua.create_table()?;
            loaded.set(module_name, module.clone())?;
            Ok(module)
        }
    }
}

#[derive(Clone, Debug, clunk::FromLuaConfig)]
struct EgData1 {
    msg: String,
    id: Option<u32>,
}

#[derive(Clone, Debug, clunk::FromLuaConfig)]
struct Things {
    label: String,
    conn: Conn,
}

#[derive(Clone, Debug, clunk::FromLuaConfig)]
struct Conn {
    host: String,
    port: u16,
}

fn main() -> Result<(), clunk_error::ClunkError> {
    println!("Hello, world!");

    let modpath = "tests/data/bad-syntax.lua";
    trace_macros!(true);

    let lc: clunk::Clunk<Things> = match clunk::Clunk::load(modpath, "things", Some("myc")) {
        Ok(lc) => lc,
        Err(e) => {
            println!("--- error ---");
            println!("{}", e);
            return Err(e);
        }
    };

    Ok(())
}
