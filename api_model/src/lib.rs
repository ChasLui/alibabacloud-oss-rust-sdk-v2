extern crate proc_macro;

mod request_field_type;
mod result_field_type;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Meta, NestedMeta, PathArguments, Type};

use self::request_field_type::*;
use self::result_field_type::*;

/// Whether a field carries `usermeta` in its `#[field(...)]` attribute.
/// Usermeta fields are `HashMap<String, String>` maps that expand to
/// `prefix + key` headers on requests and capture `prefix*` headers on results.
fn is_usermeta_field(field: &Field) -> bool {
    for attr in field.attrs.iter() {
        if !attr.path.is_ident("field") {
            continue;
        }
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            for nested in meta_list.nested.iter() {
                match nested {
                    NestedMeta::Meta(Meta::Path(p)) if p.is_ident("usermeta") => return true,
                    NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("usermeta") => {
                        return true
                    }
                    _ => {}
                }
            }
        }
    }
    false
}

/// The `rename` value of a usermeta field, used as the header prefix
/// (e.g. `x-oss-meta-`).
fn usermeta_prefix(field: &Field) -> Option<String> {
    for attr in field.attrs.iter() {
        if !attr.path.is_ident("field") {
            continue;
        }
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            for nested in meta_list.nested.iter() {
                if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                    if nv.path.is_ident("rename") {
                        if let syn::Lit::Str(lit) = &nv.lit {
                            return Some(lit.value());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Last path segment of a field type (e.g. `String` for `std::string::String`).
fn type_name(field: &Field) -> Option<String> {
    if let Type::Path(type_path) = &field.ty {
        return type_path.path.segments.last().map(|s| s.ident.to_string());
    }
    None
}

/// Whether `name` is a primitive numeric type.
fn is_numeric_type(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "usize" | "f32" | "f64"
    )
}

#[proc_macro_derive(OssRequestModel, attributes(field))]
pub fn request_model_derive(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let fields = if let syn::Data::Struct(data) = &mut input.data {
        &mut data.fields
    } else {
        return TokenStream::from(quote! {
            compile_error!("OssRequestModel can be only derived for structs");
        });
    };

    let mut header_insert_vec = vec![];
    let mut query_insert_vec = vec![];
    let mut rename_vec = vec![];

    fn is_option_type(field: &Field) -> bool {
        if let Type::Path(type_path) = &field.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        return args.args.len() == 1;
                    }
                }
            }
        }
        false
    }

    let mut usermeta_insert_vec = vec![];
    for field in fields.iter() {
        if !is_usermeta_field(field) {
            continue;
        }
        let field_ident = field.ident.as_ref().unwrap();
        let prefix = match usermeta_prefix(field) {
            Some(p) => p,
            None => {
                return TokenStream::from(quote! {
                    compile_error!(
                        "usermeta field requires a `rename` prefix, e.g. rename = \"x-oss-meta-\""
                    );
                })
            }
        };
        usermeta_insert_vec.push(quote! {
            self.#field_ident.iter().for_each(|(k, v)| {
                map.insert(format!("{}{}", #prefix, k), v.clone());
            });
        });
    }

    for field in fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();

        // `usermeta` fields expand to one header per map entry (prefix taken
        // from `rename`) and are emitted after this loop.
        if is_usermeta_field(field) {
            continue;
        }

        for attr in field.attrs.iter() {
            // only process `field` attributes
            if attr.path.is_ident("field") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("type") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    // check if field_type is one of FieldType
                                    if let Ok(field_type) = lit.value().parse::<RequestFieldType>()
                                    {
                                        // add field to corresponding field type vector
                                        match field_type {
                                            RequestFieldType::Header => &mut header_insert_vec,
                                            RequestFieldType::Query => &mut query_insert_vec,
                                        }
                                        .push(
                                            if is_option_type(field) {
                                                quote! {
                                                    if self.#field_ident.is_some() {
                                                        map.insert(
                                                            stringify!(#field_ident).to_string(),
                                                            self.#field_ident.as_ref().unwrap().to_string()
                                                        );
                                                    }
                                                }
                                            } else {
                                                // Mirror Go's `isEmptyValue`: skip empty strings,
                                                // zero numerics, and `false` so defaults do not
                                                // leak empty headers/params onto the wire.
                                                let guard = match type_name(field).as_deref() {
                                                    Some("String") => {
                                                        quote! { !self.#field_ident.is_empty() }
                                                    }
                                                    Some("bool") => quote! { self.#field_ident },
                                                    Some(name) if is_numeric_type(name) => {
                                                        quote! { self.#field_ident != 0 }
                                                    }
                                                    _ => quote! { true },
                                                };
                                                quote! {
                                                    if #guard {
                                                        map.insert(
                                                            stringify!(#field_ident).to_string(),
                                                            self.#field_ident.to_string()
                                                        );
                                                    }
                                                }
                                            },
                                        );
                                    } else {
                                        return TokenStream::from(quote! {
                                            compile_error!(
                                                "Invalid field type for field, expected one of: [header, query]"
                                            );
                                        });
                                    }
                                }
                            } else if nv.path.is_ident("rename") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    rename_vec.push(quote! {
                                        let key = stringify!(#field_ident).to_string();
                                        if map.contains_key(&key) {
                                            let value = map.remove(&key).unwrap();
                                            map.insert(#lit.to_string(), value);
                                        }
                                    });
                                }
                            } else {
                                return TokenStream::from(quote! {
                                    compile_error!("Invalid field tag, expected one of: [type, rename]");
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // if let Fields::Named(fields_named) = fields {
    //     fields_named.named.push(
    //         Field::parse_named
    //             .parse2(quote! { _headers: std::collections::HashMap<String,
    // String> })             .unwrap(),
    //     );
    //     fields_named.named.push(
    //         Field::parse_named
    //             .parse2(quote! { _querys: std::collections::HashMap<String,
    // String> })             .unwrap(),
    //     );
    // }

    let output = quote! {
        impl #struct_ident {
            /// get header map, including those from common
            pub fn header_map(&self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #( #header_insert_vec )*
                #( #rename_vec )*
                #( #usermeta_insert_vec )*
                Self::merge_map_case_insensitive(&map, &self.common.headers)
            }

            /// get query map, including those from common
            pub fn query_map(&self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #( #query_insert_vec )*
                #( #rename_vec )*
                Self::merge_map_case_insensitive(&map, &self.common.parameters)
            }

            /// add header to request.common
            pub fn add_header(&mut self, key: &str, value: &str) {
                self.common.headers.insert(key.to_string(), value.to_string());
            }

            /// add query to request.common
            pub fn add_query(&mut self, key: &str, value: &str) {
                self.common.parameters.insert(key.to_string(), value.to_string());
            }

            fn merge_map_case_insensitive(
                base: &std::collections::HashMap<String, String>,
                new: &std::collections::HashMap<String, String>,
            ) -> std::collections::HashMap<String, String> {
                let mut merge_map = base.clone();

                // {base.key.to_lowercase(): base.key}
                let key_indices: std::collections::HashMap<String, String> =
                    base.keys().map(|k| (k.to_lowercase(), k.clone())).collect();

                new.iter().for_each(|(new_key, v)| {
                    let new_lower_key = new_key.to_lowercase();
                    if let Some(base_key) = key_indices.get(&new_lower_key) {
                        // base key exists, use base key
                        merge_map.insert(base_key.clone(), v.clone());
                    } else {
                        // base key not exists, just insert
                        merge_map.insert(new_key.clone(), v.clone());
                    }
                });

                merge_map
            }
        }
    };

    TokenStream::from(output)
}

#[proc_macro_derive(OssResultModel, attributes(field))]
pub fn result_model_derive(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let fields = if let syn::Data::Struct(data) = &mut input.data {
        &mut data.fields
    } else {
        return TokenStream::from(quote! {
            compile_error!("OssResultModel can be only derived for structs");
        });
    };

    let mut header_update_vec = vec![];

    // field indent to corresponding header tag
    let mut indent_tag_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // set indent_tag_map
    for field in fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();

        for attr in field.attrs.iter() {
            // only process `field` attributes
            if attr.path.is_ident("field") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("type") {
                                if let syn::Lit::Str(_) = &nv.lit {
                                    indent_tag_map
                                        .insert(field_ident.to_string(), field_ident.to_string());
                                }
                            } else if nv.path.is_ident("rename") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    indent_tag_map
                                        .insert(field_ident.to_string(), lit.value().to_string());
                                }
                            } else if nv.path.is_ident("usermeta") {
                                // usermeta fields are handled by prefix capture below
                            } else {
                                return TokenStream::from(quote! {
                                    compile_error!("Invalid field tag, expected one of: [type, rename, usermeta]");
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // add get header statements
    for field in fields.iter() {
        let field_ident = field.ident.as_ref().unwrap();

        for attr in field.attrs.iter() {
            // only process `field` attributes
            if attr.path.is_ident("field") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("type") {
                                if let syn::Lit::Str(lit) = &nv.lit {
                                    // check if field_type is one of FieldType
                                    if let Ok(field_type) = lit.value().parse::<ResultFieldType>() {
                                        // add field to corresponding field type vector
                                        let header_tag =
                                            match indent_tag_map.get(&field_ident.to_string()) {
                                                Some(name) => name,
                                                None => unreachable!(
                                                    "all fields should have a field tag"
                                                ),
                                            };
                                        if is_usermeta_field(field) {
                                            // prefix capture: collect every header whose name
                                            // starts with `header_tag` into the map, keyed by
                                            // the remainder (e.g. x-oss-meta-author -> author).
                                            // Keys are lowercased, matching Go's unmarshalHeader.
                                            let prefix_lower = header_tag.to_lowercase();
                                            header_update_vec.push(quote! {
                                                self.#field_ident = output.headers.iter().filter_map(|(k, v)| {
                                                    k.to_lowercase()
                                                        .strip_prefix(#prefix_lower)
                                                        .map(|suffix| (suffix.to_string(), v.clone()))
                                                }).collect();
                                            });
                                            continue;
                                        }
                                        match field_type {
                                            ResultFieldType::Header => {
                                                header_update_vec.push(quote! {
                                                    self.#field_ident = output.headers.iter().find_map(|(k, v)| {
                                                        if k.as_str().eq_ignore_ascii_case(#header_tag) {
                                                            Some(v.parse().unwrap()) // `FromStr` trait implemented
                                                        } else {
                                                            None
                                                        }
                                                    });
                                                })
                                            }
                                        }
                                    } else {
                                        return TokenStream::from(quote! {
                                            compile_error!("Invalid field type for field, expected one of: [header]");
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let output = quote! {
        impl #struct_ident {
            /// get header map, including those from common
            pub fn update_result(&mut self, output: &OperationOutput) {
                // modify common
                self.common.status = output.status.clone();
                self.common.headers = output.headers.clone();

                // modify other fields
                #( #header_update_vec )*
            }
        }
    };

    TokenStream::from(output)
}
