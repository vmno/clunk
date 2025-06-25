use mlua::Error as LuaError;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClunkError {
    /// If a module in the loaded packages table (package.loaded) is not a table, this will
    /// get thrown
    #[error("package.loaded.{0} exists but is not a table")]
    LoadedModuleNotTable(String),

    /// if the config data being parsed into a rust struct is not a table, this gets thrown
    #[error("failed to convert config data into `{to}` because it has the type `{_type}`")]
    ConfigDataNotTable { to: String, _type: String },

    /// Converting from a lua value to a rust value failed
    #[error("Failed to convert {from} to {to} for field {field}")]
    TypeConversionFailed {
        from: String,
        to: String,
        field: String,
    },

    /// When converting a lua table to a rust struct, a field was missing from the lua table
    #[error("Missing field {field} in table")]
    TableMissingField { field: String },

    /// if the lua config data has a field that is a table, and that table could not be
    /// parsed/converted properly then this error will be thrown
    #[error("Failed to convert nested table field ({field}) in {parent_field}: {msg}")]
    TableNestedFieldError {
        parent_field: String,
        field: String,
        msg: String,
    },

    /// Generic error from std::io
    #[error("std::io error: {0}")]
    StdIoErr(#[from] std::io::Error),

    // basically any error from mlua
    #[error("Lua error: {0}")]
    LuaErr(LuaError),
}

impl From<LuaError> for ClunkError {
    fn from(e: LuaError) -> Self {
        // we catch some of these specific errors to change them into their own types
        match e {
            // in this case the table has a field that is a table and that field could not be
            // parsed
            LuaError::FromLuaConversionError { from, to, message }
                if message.is_some()
                    && message.as_ref().unwrap().contains("nested table field") =>
            {
                // ugly but whatever. we'll just extract the parent field and field from the
                // message
                let parent_field =
                    message.clone().unwrap()[message.as_ref().unwrap().find("parent=`").unwrap()
                        ..(message.as_ref().unwrap().rfind("`)").unwrap() + 1)]
                        .to_string()
                        .replace("parent=", "");

                let field =
                    message.clone().unwrap()[message.as_ref().unwrap().find("field `").unwrap()
                        ..(message.as_ref().unwrap().rfind("`").unwrap() + 1)]
                        .to_string()
                        .replace("field ", "");

                return ClunkError::TableNestedFieldError {
                    parent_field,
                    field,
                    msg: message.unwrap(),
                };
            }

            // these are instances where a table field was missing or an incorrect type in the lua
            // config and everything else seems fine
            LuaError::FromLuaConversionError { from, to, message }
                if message.is_some()
                    && message.as_ref().unwrap().contains("table is missing field") =>
            {
                let field = message.clone().unwrap()[message.as_ref().unwrap().find("`").unwrap()
                    ..(message.as_ref().unwrap().rfind("`").unwrap() + 1)]
                    .to_string();
                //println!("field: {}", field);
                //println!("message: {:?}", message);
                //println!("from: {}", from);
                //println!("to: {}", to);

                return ClunkError::TableMissingField { field };
            }

            // in this case the table field is the wrong type
            LuaError::FromLuaConversionError { from, to, message }
                if message.is_some() && message.as_ref().unwrap().contains("wrong type") =>
            {
                let re = Regex::new(r".+`(\w+)`.+expected: (\w+)").unwrap();
                let caps = re.captures(message.as_ref().unwrap()).unwrap();
                let (field, expected) = (
                    caps.get(1).unwrap().as_str().to_string(),
                    caps.get(2).unwrap().as_str().to_string(),
                );

                return ClunkError::TypeConversionFailed {
                    from: from.to_string(),
                    to: to.to_string(),
                    field,
                };
            }

            // in this case the table that should be config data is not a table but another type
            LuaError::FromLuaConversionError { from, to, message }
                if message.is_some()
                    && message.as_ref().unwrap().contains("expected a Lua table") =>
            {
                return ClunkError::ConfigDataNotTable {
                    _type: from.to_string(),
                    to: to.to_string(),
                };
            }
            // the rest of the errors can just be returned as a generic LuaErr
            _ => ClunkError::LuaErr(e),
        }
    }
}

pub type ClunkResult<T> = Result<T, ClunkError>;
