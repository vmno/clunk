use thiserror::Error;

pub enum LuckError {
    /// Converting from a lua value to a rust value failed
    TypeConversionFailed {
        from: &'static str,
        to: &'static str,
        field: &'static str,
    },
    /// When converting a lua table to a rust struct, a field was missing from the lua table
    TableMissingField { field: &'static str },
}
