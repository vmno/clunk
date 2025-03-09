//
use mlua;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index,
};

use clunk_error;

/// `FromLuaConfig` derive macro that can be used to generate a `FromLua` impl for a user struct.
/// Allows a lua table to be converted into a rust struct.
///
#[proc_macro_derive(FromLuaConfig)]
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
    // construct the fields for the struct from the lua table fields.
    let fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        quote! {
            #ident: table.get(stringify!(#ident)).or_else(|e| {
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
                            from: "shit",
                            to: stringify!(#ty),
                            message: Some(format!("{} --- {}", stringify!(#ident), e)),
                        })
                    }
                }
            })?//.unwrap()
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
