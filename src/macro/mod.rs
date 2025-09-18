//
//#![feature(trace_macros)]
use mlua;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index,
};

use clunk_error;

//trace_macros!(true);

// This handy macro can be used to debug a proc macro by writing out the expanded code to a file.
// I got it from github.com/lescas/tealr
// https://github.com/lenscas/tealr/blob/c5a317994fbeb48a67ea849674b7e010cd2c8aa4/tealr_derive/src/from_to_lua.rs#L11
//
#[allow(dead_code)]
fn debug_macro(ts: TokenStream) -> TokenStream {
    let hopefully_unique = {
        use ::std::hash::*;
        let hasher = &mut RandomState::new().build_hasher();
        hasher.finish()
    };

    //FEEL FREE TO TWEAK THIS DEFAULT PATH (e.g., your target dir)
    let mut debug_macros_dir = ::std::path::PathBuf::from("/tmp");
    //let mut debug_macros_dir = ::std::path::PathBuf::from("./");
    std::fs::create_dir_all(&debug_macros_dir).unwrap();
    let file_name = &{
        debug_macros_dir.push(format!("{:016x}.rs", hopefully_unique));
        debug_macros_dir.into_os_string().into_string().unwrap()
    };
    std::fs::write(file_name, ts.to_string()).unwrap();
    quote!(::core::include! { #file_name })
}

/// `FromLuaConfig` derive macro that can be used to generate a `FromLua` impl for a user struct.
/// Allows a lua table to be converted into a rust struct.
/// 
/// Supports the `#[ignore_field]` attribute on fields to skip deserialization and use Default::default().
///
#[proc_macro_derive(FromLuaConfig, attributes(ignore_field))]
pub fn from_lua_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // the name of the struct
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // input.data holds the storage for our struct
    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    // Collect types of fields that have #[ignore_field] attribute for Default bounds
    let ignored_field_types: Vec<_> = fields.iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| attr.path().is_ident("ignore_field"))
        })
        .map(|field| &field.ty)
        .collect();

    // Create where clause with Default bounds for ignored fields
    let mut where_clause = where_clause.cloned();
    if !ignored_field_types.is_empty() {
        let default_bounds = ignored_field_types.iter().map(|ty| {
            quote! { #ty: Default }
        });
        
        if where_clause.is_none() {
            where_clause = Some(parse_quote! { where #(#default_bounds),* });
        } else {
            let existing_predicates = &where_clause.as_ref().unwrap().predicates;
            where_clause = Some(parse_quote! { 
                where #existing_predicates, #(#default_bounds),* 
            });
        }
    }

    let fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        // Check if field has #[ignore_field] attribute
        let has_ignore = field.attrs.iter().any(|attr| {
            attr.path().is_ident("ignore_field")
        });

        if has_ignore {
            // For ignored fields, use Default::default()
            quote! {
                #ident: Default::default()
            }
        } else {
            // Normal field processing
            quote! {
            #ident: {
                let value = table.get(stringify!(#ident)).or_else(|e| {
                    match e {
                        // if from is nil then its likely the required field is missing from the
                        // lua table
                        mlua::Error::FromLuaConversionError {
                            from: from, to: to, message: Some(msg)
                        } if from == "nil" => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from,
                                to: stringify!(#ty),
                                message: Some(
                                    format!(
                                        "table is missing field `{}`",
                                        stringify!(#ident),
                                    )
                                ),
                            });
                        }
                        // cases where the field has the wrong type, e.g., providing a string when
                        // the rust struct expects an integer
                        mlua::Error::FromLuaConversionError {
                            from: from, to: to, message: Some(msg)
                        } => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from,
                                to: stringify!(#ty),
                                message: Some(
                                    format!(
                                        "field `{}` is the wrong type, expected: {}",
                                        stringify!(#ident),
                                        stringify!(#ty)
                                    )
                                ),
                            });
                        }
                        // generic error case, there are probabaly other specific mlua cases that
                        // I should catch but I haven't seen others aside from the above
                        _ => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "",
                                to: stringify!(#ty),
                                message: Some(format!("{} --- {}", stringify!(#ident), e)),
                            })
                        }
                    }
                })?;

                // try to convert the value to the target type
                // this is where nested struct conversion happens and we'll try to provide a better
                // error message in cases where the inner struct conversion fails
                <#ty as mlua::FromLua>::from_lua(value, lua).or_else(|e| {
                    match &e {
                        // if there's an issue with a nested struct, we should see some relevant
                        // text in the error message. this is ugly but whatever
                        mlua::Error::FromLuaConversionError { from, to, message } if message.clone().unwrap_or_else(|| format!("" )).contains("table is missing field") => {
                            let msg = message.clone().unwrap_or_else(|| format!("{}", e));
                            return Err(mlua::Error::FromLuaConversionError {
                                from,
                                to: stringify!(#ty),
                                message: Some(
                                    format!(
                                        "nested table field (parent=`{}`) error: {}",
                                        stringify!(#ident),
                                        msg
                                    )
                                ),
                            });
                        },
                        // if from is nil then its likely the required field is missing from the
                        // lua table
                        mlua::Error::FromLuaConversionError { from, to, message } if from.clone() == "nil" => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from,
                                to: stringify!(#ty),
                                message: Some(
                                    format!(
                                        "table is missing field `{}`",
                                        stringify!(#ident),
                                    )
                                ),
                            });
                        },
                        // generic type conversion error
                        mlua::Error::FromLuaConversionError { from, to, message } if from.clone() != to.clone() => {
                            return Err(mlua::Error::FromLuaConversionError {
                                from,
                                to: stringify!(#ty),
                                message: Some(
                                    format!(
                                        "field `{}` is the wrong type, expected: {}",
                                        stringify!(#ident),
                                        stringify!(#ty)
                                    )
                                ),
                            });
                        },

                        mlua::Error::FromLuaConversionError { from, to, message } => {
                            let error_message = message.clone().unwrap_or_else(|| format!("{}", e));
                            Err(mlua::Error::FromLuaConversionError {
                                from: from.clone(),
                                to: to.clone(),
                                message: Some(format!(
                                    "nested field error: parent=`{}`, field=`{}`, message={}",
                                    stringify!(#ident),
                                    // I don't think we'll know the exact field name in the nested struct,
                                    // but we'll include the error message which should have it
                                    "unknown",
                                    error_message
                                )),
                            })
                        },
                        _ => Err(e)
                    }
                })?
            }
        }
        }
    });

    // construct the FromLua impl, explicit check that the lua variable is a table
    let expanded = quote! {
        impl<'lua> #impl_generics mlua::FromLua<'lua> for #name #ty_generics #where_clause {
            fn from_lua(lua_value: mlua::Value, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
            //fn from_lua(lua_value: mlua::Value, lua: &'lua mlua::Lua) -> Result<Self> {
                let table = match lua_value.as_table() {
                    Some(t) => t,
                    None => {
                        return Err(mlua::Error::FromLuaConversionError {
                            from: lua_value.type_name(),
                            to: stringify!(#name),
                            message: Some(String::from("expected a Lua table")),
                        });
                    }
                };

                Ok(#name {
                    #(#fields),*
                })
            }
        }
    };

    expanded.into()
}
